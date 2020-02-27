use crate::{
    extractor::{auth::TokenAuth, if_match::IfMatch, ip::Ip},
    model::demonlist::{
        record::{FullRecord, PatchRecord, RecordPagination, RecordStatus, Submission},
        submitter::Submitter,
    },
    permissions::Permissions,
    state::{audit_connection, PointercrateState},
    util::HttpResponseBuilderExt,
    ApiResult,
};
use actix_web::{
    web::{Json, Path, Query},
    HttpResponse,
};
use actix_web_codegen::{delete, get, patch, post};

#[get("/")]
pub async fn paginate(
    user: ApiResult<TokenAuth>, state: PointercrateState, mut pagination: Query<RecordPagination>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;

    // Non authenticated access and access by users without ExtendedAccess perms cannot see non-approved
    // records
    let can_see_all_records = if let Ok(TokenAuth(user)) = user {
        user.inner().extended_list_access()
    } else {
        false
    };

    if !can_see_all_records {
        match pagination.0.status {
            // empty response if we filter by a status != approved and we cant see those records
            Some(status) if status != RecordStatus::Approved => return Ok(HttpResponse::Ok().json(Vec::<()>::new())),
            _ => pagination.0.status = Some(RecordStatus::Approved),
        }
    }

    let records = pagination.page(&mut connection).await?;

    let (max_id, min_id) = FullRecord::extremal_record_ids(&mut connection).await?;

    pagination_response!(records, pagination, min_id, max_id, before_id, after_id, id)
}

#[post("/")]
pub async fn submit(
    Ip(ip): Ip, user: ApiResult<TokenAuth>, submission: Json<Submission>, state: PointercrateState,
) -> ApiResult<HttpResponse> {
    let mut connection = state.transaction().await?;

    // NOTE: don't abort if authentication fails! We might not need it!
    // This prevents invalid auth data in cookies to interfere with record submission

    let shall_ratelimit = user.as_ref().map(|user| !user.0.inner().list_team_member()).unwrap_or(true);

    // only members of the list team can directly add approved records, or add records without video
    if submission.status != RecordStatus::Submitted || submission.video.is_none() {
        // do abort if it fails here!
        let user = user?.0;

        user.inner().require_permissions(Permissions::ListHelper)?;
        audit_connection(&mut connection, user.inner()).await?; // might as well
    }

    let ratelimiter = state.ratelimits.prepare(ip);

    let submitter = Submitter::by_ip_or_create(ip, &mut connection, Some(ratelimiter)).await?;

    let record = if shall_ratelimit {
        FullRecord::create_from(submitter, submission.into_inner(), &mut connection, Some(ratelimiter)).await?
    } else {
        FullRecord::create_from(submitter, submission.into_inner(), &mut connection, None).await?
    };

    connection.commit().await?;

    let response = HttpResponse::Created()
        .header("Location", format!("/api/v1/records/{}/", record.id))
        .json_with_etag(&record);

    // spawn background task to validate record
    if record.status == RecordStatus::Submitted {
        actix_rt::spawn(record.validate(state));
    }

    Ok(response)
}

#[get("/{record_id}/")]
pub async fn get(user: ApiResult<TokenAuth>, state: PointercrateState, record_id: Path<i32>) -> ApiResult<HttpResponse> {
    let mut connection = state.connection().await?;
    let record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if record.status != RecordStatus::Approved {
        user?.0.inner().require_permissions(Permissions::ExtendedAccess)?;
    }

    Ok(HttpResponse::Ok().json_with_etag(&record))
}

#[patch("/{record_id}/")]
pub async fn patch(
    TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, record_id: Path<i32>, data: Json<PatchRecord>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.transaction().await?;

    user.inner().require_permissions(Permissions::ListHelper)?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let mut record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&record)?;

    record = record.apply_patch(data.into_inner(), &mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::Ok().json_with_etag(&record))
}

#[delete("/{record_id}/")]
pub async fn delete(
    TokenAuth(user): TokenAuth, if_match: IfMatch, state: PointercrateState, record_id: Path<i32>,
) -> ApiResult<HttpResponse> {
    let mut connection = state.transaction().await?;

    user.inner().require_permissions(Permissions::ListAdministrator)?;

    // FIXME: prevent lost updates by using SELECT ... FOR UPDATE
    let record = FullRecord::by_id(record_id.into_inner(), &mut connection).await?;

    if_match.require_etag_match(&record)?;

    record.delete(&mut connection).await?;

    connection.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

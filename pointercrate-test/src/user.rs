use rocket::local::asynchronous::Client;
use sqlx::{PgConnection, Pool, Postgres};
use sqlx::pool::PoolConnection;
use pointercrate_core::permission::{Permission, PermissionsManager};
use pointercrate_core::pool::PointercratePool;
use pointercrate_user::{ADMINISTRATOR, AuthenticatedUser, MODERATOR, Registration};
use pointercrate_user_pages::account::AccountPageConfig;
use crate::TestClient;

pub async fn system_user_with_perms(perm: Permission, connection: &mut PgConnection) -> AuthenticatedUser {
    let user = AuthenticatedUser::register(
        Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        },
        &mut *connection,
    )
        .await
        .unwrap();

    sqlx::query!(
        "UPDATE members SET permissions = $2::INTEGER::BIT(16) WHERE member_id = $1",
        user.inner().id,
        perm.bit() as i16
    )
        .execute(connection)
        .await
        .unwrap();

    user
}

pub async fn add_normal_user(connection: &mut PgConnection) -> AuthenticatedUser {
    AuthenticatedUser::register(
        Registration {
            name: "Patrick".to_string(),
            password: "bad password".to_string(),
        },
        connection,
    )
        .await
        .unwrap()
}
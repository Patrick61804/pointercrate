table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    creators (demon, creator) {
        demon -> Citext,
        creator -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    demons (name) {
        name -> Citext,
        position -> Int2,
        requirement -> Int2,
        video -> Nullable<Varchar>,
        verifier -> Int4,
        publisher -> Int4,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    members (member_id) {
        member_id -> Int4,
        name -> Text,
        password_hash -> Text,
        permissions -> BitString,
        display_name -> Nullable<Text>,
        youtube_channel -> Nullable<Varchar>,
        nationality -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    nationalities (nation) {
        nation -> Text,
        iso_country_code -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    players (id) {
        id -> Int4,
        name -> Citext,
        banned -> Bool,
        nationality -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    records (id) {
        id -> Int4,
        progress -> Int2,
        video -> Nullable<Varchar>,
        status_ -> Record_status,
        player -> Int4,
        submitter -> Int4,
        demon -> Citext,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::model::record::Record_status;
    use crate::citext::Citext;
    use crate::bitstring::BitString;

    submitters (submitter_id) {
        submitter_id -> Int4,
        ip_address -> Inet,
        banned -> Bool,
    }
}

joinable!(creators -> demons (demon));
joinable!(creators -> players (creator));
joinable!(records -> demons (demon));
joinable!(records -> players (player));
joinable!(records -> submitters (submitter));

allow_tables_to_appear_in_same_query!(
    creators,
    demons,
    members,
    nationalities,
    players,
    records,
    submitters,
);

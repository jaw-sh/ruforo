table! {
    ip (id) {
        id -> Int4,
        address -> Inet,
        first_seen_on -> Timestamp,
        last_seen_on -> Timestamp,
    }
}

table! {
    ugc (id) {
        id -> Int4,
        ugc_revision_id -> Nullable<Int4>,
    }
}

table! {
    ugc_revisions (id) {
        id -> Int4,
        ugc_id -> Int4,
        ip_id -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
        created_at -> Timestamp,
        content -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Int4,
        created_on -> Nullable<Timestamp>,
        name -> Text,
    }
}

joinable!(ugc_revisions -> ip (ip_id));
joinable!(ugc_revisions -> users (user_id));

allow_tables_to_appear_in_same_query!(ip, ugc, ugc_revisions, users,);

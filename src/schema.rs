table! {
    tf_ip (ip_id) {
        ip_id -> Int4,
        ip -> Inet,
        first_seen_on -> Timestamp,
        last_seen_on -> Timestamp,
    }
}

table! {
    tf_ugc (ugc_id) {
        ugc_id -> Int4,
        ugc_revision_id -> Int4,
    }
}

table! {
    tf_ugc_revisions (ugc_revision_id) {
        ugc_revision_id -> Int4,
        ugc_id -> Int4,
        ip_id -> Nullable<Int4>,
        user_id -> Nullable<Int4>,
        created_at -> Timestamp,
        content -> Nullable<Text>,
    }
}

table! {
    tf_users (user_id) {
        user_id -> Int4,
        created_on -> Nullable<Timestamp>,
        name -> Text,
    }
}

joinable!(tf_ugc_revisions -> tf_ip (ip_id));
joinable!(tf_ugc_revisions -> tf_users (user_id));

allow_tables_to_appear_in_same_query!(
    tf_ip,
    tf_ugc,
    tf_ugc_revisions,
    tf_users,
);

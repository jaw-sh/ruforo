table! {
    ip (id) {
        id -> Int4,
        address -> Inet,
        first_seen_at -> Timestamp,
        last_seen_at -> Timestamp,
    }
}

table! {
    posts (id) {
        id -> Int4,
        thread_id -> Int4,
        ugc_id -> Int4,
        user_id -> Nullable<Int4>,
        created_at -> Timestamp,
    }
}

table! {
    sessions (id) {
        id -> Bpchar,
        data -> Json,
        expires_at -> Timestamp,
    }
}

table! {
    threads (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        created_at -> Timestamp,
        title -> Text,
        subtitle -> Nullable<Text>,
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
        created_at -> Timestamp,
        name -> Text,
        password -> Text,
    }
}

joinable!(posts -> threads (thread_id));
joinable!(posts -> ugc (ugc_id));
joinable!(posts -> users (user_id));
joinable!(threads -> users (user_id));
joinable!(ugc_revisions -> ip (ip_id));
joinable!(ugc_revisions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    ip,
    posts,
    sessions,
    threads,
    ugc,
    ugc_revisions,
    users,
);

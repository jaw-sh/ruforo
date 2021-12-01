table! {
    posts (id) {
        id -> Int8,
        title -> Varchar,
        body -> Text,
        post_date -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int8,
        username -> Varchar,
        password -> Varchar,
        join_date -> Timestamp,
        email -> Nullable<Varchar>,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);

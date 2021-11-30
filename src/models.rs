use chrono::NaiveDateTime;
use serde::Serialize;

use crate::schema::posts;
use crate::schema::users;

#[derive(Queryable)]
pub struct Ugc {
    pub ugc_id: i64,
    pub ugc_revision_id: i64
}

#[derive(Queryable)]
pub struct UgcRevision {
    pub ugc_revision_id: i64,
    pub ugc_id: i64,
    pub ip_id: i64,
    pub user_id: i64,
    pub created_at: NaiveDateTime,
    pub content: String
}

// #[derive(Queryable)]
// struct Board {
//     id: u32,
//     name: String,
//     description: String,
// }

#[derive(Serialize, Queryable)]
pub struct User {
    pub user_id: i64,
    pub created_on: NaiveDateTime,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "tf_ugc"]
pub struct NewUgc<'a> {
    pub user_id: i64,
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(Insertable)]
#[table_name = "tf_ugc_revisions"]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub created_at: diesel::dsl::now,
    pub content: Option<&'a str>,
}

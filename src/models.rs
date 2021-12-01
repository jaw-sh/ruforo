use chrono::NaiveDateTime;
use serde::Serialize;

use crate::schema::users;
use crate::schema::ugc;
use crate::schema::ugc_revisions;

#[derive(Identifiable, Queryable, PartialEq)]
#[table_name = "ugc"]
pub struct Ugc {
    pub id: i32,
    pub ugc_revision_id: Option<i32>,
}

#[derive(Associations, Identifiable, Queryable, PartialEq)]
#[belongs_to(Ugc, foreign_key = "ugc_id")]
pub struct UgcRevision {
    pub id: i32,
    pub ugc_id: i32,
    pub ip_id: Option<i32>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub content: Option<String>,
}

// #[derive(Queryable)]
// struct Board {
//     id: u32,
//     name: String,
//     description: String,
// }

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub name: String,
    pub password: String,
}

#[derive(Insertable)]
#[table_name = "ugc"]
pub struct NewUgc {
    pub ugc_revision_id: i32,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub created_at: diesel::dsl::now,
    pub name: &'a str,
    pub password: &'a str,
}

#[derive(Insertable)]
#[table_name = "ugc_revisions"]
pub struct NewUgcRevision {
    pub user_id: Option<i32>,
    pub ip_id: Option<i32>,
    pub content: Option<String>,
}

#[derive(Insertable)]
#[table_name = "ugc_revisions"]
pub struct NewUgcRevisionWithContext {
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub ip_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub content: Option<String>,
}

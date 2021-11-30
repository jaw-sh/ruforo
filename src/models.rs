use chrono::NaiveDateTime;
use serde::Serialize;

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

#[derive(Serialize, Queryable)]
pub struct User {
    pub user_id: i32,
    pub created_on: NaiveDateTime,
    pub name: String,
}

#[derive(Insertable)]
#[table_name = "ugc"]
pub struct NewUgc {
    pub ugc_revision_id: i32,
}

#[derive(Insertable)]
#[table_name = "ugc_revisions"]
pub struct NewUgcRevision {
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub ip_id: Option<i32>,
    pub created_at: diesel::dsl::now,
    pub content: Option<String>,
}

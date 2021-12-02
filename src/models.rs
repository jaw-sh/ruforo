use chrono::NaiveDateTime;
//use serde::Serialize;

use crate::schema::posts;
use crate::schema::threads;
use crate::schema::ugc;
use crate::schema::ugc_revisions;
use crate::schema::users;

#[derive(Identifiable, Queryable, PartialEq)]
#[table_name = "ugc"]
pub struct Ugc {
    pub id: i32,
    pub ugc_revision_id: Option<i32>,
    pub first_revision_at: NaiveDateTime,
    pub last_revision_at: NaiveDateTime,
}

#[derive(Associations, Identifiable, Queryable, PartialEq)]
#[belongs_to(Ugc, foreign_key = "ugc_id")]
#[table_name = "ugc_revisions"]
pub struct UgcRevision {
    pub id: i32,
    pub ugc_id: i32,
    pub ip_id: Option<i32>,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub content: Option<String>,
}

#[derive(Associations, Identifiable, Queryable, PartialEq)]
#[belongs_to(Thread, foreign_key = "thread_id")]
#[belongs_to(Ugc, foreign_key = "ugc_id")]
#[belongs_to(User, foreign_key = "user_id")]
#[table_name = "posts"]
pub struct Post {
    pub id: i32,
    pub thread_id: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Associations, Identifiable, Queryable, PartialEq)]
#[table_name = "threads"]
pub struct Thread {
    pub id: i32,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub title: String,
    pub subtitle: Option<String>,
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub name: String,
    pub password: String,
}

// Inserts
#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost {
    pub thread_id: i32,
    pub ugc_id: i32,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "threads"]
pub struct NewThread {
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub title: String,
    pub subtitle: Option<String>,
}

#[derive(Insertable)]
#[table_name = "ugc"]
pub struct NewUgc {
    pub first_revision_at: NaiveDateTime,
    pub last_revision_at: NaiveDateTime,
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

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub created_at: diesel::dsl::now,
    pub name: &'a str,
    pub password: &'a str,
}

// Renderables
pub struct RenderUgc<'a> {
    pub ugc: &'a Ugc,
    pub revision: &'a UgcRevision,
}

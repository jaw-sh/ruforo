use chrono::NaiveDateTime;
use serde::Serialize;

use crate::schema::posts;
use crate::schema::users;

#[derive(Queryable)]
pub struct Post {
	pub id: i64,
	pub title: String,
	pub body: String,
}

// #[derive(Queryable)]
// struct Board {
// 	id: u32,
// 	name: String,
// 	description: String,
// }

#[derive(Serialize, Queryable)]
pub struct User {
	pub id: i64,
	pub username: String,
	pub join_date: NaiveDateTime,
	pub email: Option<String>,
	// pub join_date: diesel::sql_types::Timestamp,
}

#[derive(Insertable)]
#[table_name = "posts"]
pub struct NewPost<'a> {
	pub title: &'a str,
	pub body: &'a str,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
	pub username: &'a str,
	pub join_date: diesel::dsl::now,
	pub email: Option<&'a str>,
}

#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod models;
pub mod schema;

use argon2::{password_hash::SaltString, Argon2};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2;
use std::sync::Mutex;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub struct BigChungus {
    pub val: Mutex<i32>,
    pub start_time: NaiveDateTime,
}

impl BigChungus {
    pub fn new() -> Self {
        BigChungus {
            val: Mutex::new(32),
            start_time: chrono::Utc::now().naive_utc(),
        }
    }
}

pub struct MyAppData<'key> {
    pub salt: SaltString,
    pub argon2: Argon2<'key>,
    pub pool: DbPool,
    pub cache: BigChungus,
}

impl MyAppData<'_> {
    pub fn new(salt: SaltString) -> Self {
        let manager = new_db_manager();
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        Self {
            salt,
            argon2: Argon2::default(),
            pool,
            cache: BigChungus::new(),
        }
    }
}

fn new_db_manager() -> r2d2::ConnectionManager<PgConnection> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    r2d2::ConnectionManager::<PgConnection>::new(database_url)
}

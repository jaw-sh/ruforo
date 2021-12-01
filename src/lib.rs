#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

use argon2::{password_hash::SaltString, Argon2};
use diesel::prelude::*;
use diesel::r2d2;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub struct MyAppData<'key> {
    pub salt: SaltString,
    pub argon2: Argon2<'key>,
}

impl MyAppData<'_> {
    pub fn new(salt: SaltString) -> Self {
        Self {
            salt,
            argon2: Argon2::default(),
        }
    }
}


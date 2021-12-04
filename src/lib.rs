use argon2::{password_hash::SaltString, Argon2};
use chrono::NaiveDateTime;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;
use uuid::Uuid;

pub type SessionMap = RwLock<HashMap<Uuid, Session>>;

#[derive(Copy, Clone)]
pub struct Session {
    pub user_id: i32,
    pub expire: NaiveDateTime,
}

pub struct BigChungus {
    pub val: RwLock<i32>,
    pub start_time: NaiveDateTime,
    pub sessions: SessionMap,
}

impl BigChungus {
    pub fn new() -> Self {
        BigChungus {
            val: RwLock::new(32),
            start_time: chrono::Utc::now().naive_utc(),
            sessions: RwLock::new(HashMap::new()),
        }
    }
}

pub struct MainData<'key> {
    pub salt: SaltString,
    pub argon2: Argon2<'key>,
    pub pool: DatabaseConnection,
    pub cache: BigChungus,
}

impl MainData<'_> {
    pub fn new(pool: DatabaseConnection, salt: SaltString) -> Self {
        Self {
            salt,
            argon2: Argon2::default(),
            pool,
            cache: BigChungus::new(),
        }
    }
}

pub async fn new_db_pool() -> Result<DatabaseConnection, DbErr> {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true);

    Database::connect(opt).await
}

use crate::session::{get_sess, reload_session_cache};
use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;

static DB_POOL: OnceCell<DatabaseConnection> = OnceCell::new();

#[inline(always)]
pub fn get_db_pool() -> &'static DatabaseConnection {
    unsafe { DB_POOL.get_unchecked() }
}

/// Opens the database URL and initializes the DB_POOL static.
pub async fn init_db(database_url: String) -> &'static DatabaseConnection {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(1))
        .idle_timeout(Duration::from_secs(1))
        .sqlx_logging(true);

    let pool = Database::connect(opt)
        .await
        .expect("Database connection was not established.");
    DB_POOL.set(pool).unwrap();

    reload_session_cache(get_sess())
        .await
        .expect("failed to reload_session_cache");

    DB_POOL
        .get()
        .expect("DatabaseConnection in DB_POOL failed in init_db()")
}

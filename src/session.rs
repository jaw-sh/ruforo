use crate::orm;
use crate::orm::sessions::Entity as Sessions;
use actix_web::{get, web, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use chrono::{NaiveDateTime, Utc};
use sea_orm::{entity::*, query::*, ConnectOptions, Database, DatabaseConnection, DbErr};
use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;
use uuid::Uuid;

lazy_static! {
    static ref SALT: SaltString = {
        dotenv::dotenv().ok();
        let salt = match std::env::var("SALT") {
            Ok(v) => v,
            Err(e) => {
                let salt = SaltString::generate(&mut OsRng);
                panic!(
                    "Missing SALT ({:?}) here's a freshly generated one: {}",
                    e,
                    salt.as_str()
                );
            }
        };
        SaltString::new(&salt).unwrap()
    };
}

pub async fn init_data<'key>() -> MainData<'key> {
    let pool = new_db_pool().await.expect("Failed to create pool");
    let mut data = MainData::new(pool, &SALT);
    reload_session_cache(&data.pool, &mut data.cache.sessions)
        .await
        .expect("failed to reload_session_cache");
    data
}

#[derive(Copy, Clone)]
pub struct Session {
    pub user_id: i32,
    pub expires_at: NaiveDateTime,
}

#[derive(Copy, Clone)]
pub struct SessionWithUuid {
    pub uuid: Uuid,
    pub session: Session,
}

pub type SessionMap = RwLock<HashMap<Uuid, Session>>;
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
impl Default for BigChungus {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MainData<'key> {
    pub argon2: Argon2<'key>,
    pub pool: DatabaseConnection,
    pub cache: BigChungus,
}

impl<'key> MainData<'key> {
    pub fn new(pool: DatabaseConnection, salt: &'key SaltString) -> Self {
        MainData {
            argon2: Argon2::new_with_secret(
                salt.as_bytes(),
                argon2::Algorithm::default(),
                argon2::Version::default(),
                argon2::Params::default(),
            )
            .expect("failed to create argon2"),
            pool,
            cache: BigChungus::new(),
        }
    }
}

/// Accepts the actix_web Cookies jar and returns a session, if authentication can be found and made.
pub fn authenticate_by_cookie(
    ses_map: &SessionMap,
    cookies: &actix_session::Session,
) -> Option<SessionWithUuid> {
    match cookies.get::<String>("token") {
        Ok(Some(token)) => authenticate_by_uuid_string(ses_map, token),
        _ => None,
    }
}

/// Accepts a UUID as a string and returns a session, if the UUID can parse and authenticate.
pub fn authenticate_by_uuid_string(ses_map: &SessionMap, uuid: String) -> Option<SessionWithUuid> {
    match Uuid::parse_str(&uuid) {
        Ok(uuid) => authenticate_by_uuid(ses_map, uuid),
        _ => None,
    }
}

/// Accepts a uuid::Uuid type and returns a session if the token can authenticate.
pub fn authenticate_by_uuid(ses_map: &SessionMap, uuid: Uuid) -> Option<SessionWithUuid> {
    ses_map
        .read()
        .unwrap()
        .get(&uuid)
        .map(|session| SessionWithUuid {
            uuid,
            session: session.to_owned(),
        })
}

async fn new_db_pool() -> Result<DatabaseConnection, DbErr> {
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

pub async fn new_session(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
    user_id: i32,
) -> Result<Uuid, DbErr> {
    let expires_at = chrono::Utc::now().naive_utc();
    let ses = Session {
        user_id,
        expires_at,
    };
    let mut uuid;
    loop {
        uuid = Uuid::new_v4();
        let ses_map = &mut *ses_map.write().unwrap();
        if let hash_map::Entry::Vacant(e) = ses_map.entry(uuid) {
            e.insert(ses);
            break;
        }
    }

    let session = orm::sessions::ActiveModel {
        id: Set(uuid.to_string()),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
    };
    Sessions::insert(session).exec(db).await?;

    Ok(uuid)
}

/// copies a session out of the mutex protected hashmap
pub fn get_session(ses_map: &SessionMap, uuid: &Uuid) -> Option<Session> {
    // TODO add expiration checking
    ses_map
        .read()
        .unwrap()
        .get(uuid)
        .map(|uuid| uuid.to_owned())
}

/// use get_session instead unless you have a really good reason to talk to the DB
pub async fn get_session_from_db(
    db: &DatabaseConnection,
    uuid: &Uuid,
) -> Result<Option<orm::sessions::Model>, DbErr> {
    Sessions::find_by_id(uuid.to_string()).one(db).await
}

pub async fn reload_session_cache(
    db: &DatabaseConnection,
    ses_map: &mut SessionMap,
) -> Result<(), DbErr> {
    let results = Sessions::find().all(db).await?;
    let mut ses_map = ses_map.write().unwrap();
    for result in results {
        ses_map.insert(
            Uuid::parse_str(&result.id).map_err(|e| {
                log::error!("{}", e);
                DbErr::Custom(e.to_string())
            })?,
            Session {
                user_id: result.user_id,
                expires_at: result.expires_at,
            },
        );
    }
    Ok(())
}

pub async fn remove_session(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
    uuid: Uuid,
) -> Result<Option<Session>, DbErr> {
    use crate::orm::sessions;

    let ses_map = &mut *ses_map.write().unwrap();
    if ses_map.contains_key(&uuid) {
        log::error!("remove_session: Deleting UUID found in ses map.");
        sessions::Entity::delete_many()
            .filter(sessions::Column::Id.eq(uuid.to_string()))
            .exec(db)
            .await?;
        Ok(ses_map.remove(&uuid))
    } else {
        log::error!("remove_session: UUID not found in ses map.");
        Ok(None)
    }
}

pub async fn task_expire_sessions(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
) -> Result<(usize, Vec<Uuid>), DbErr> {
    use crate::orm::sessions;

    // allocate memory up front with a read lock to minimize write lock
    // this can be reduced since expiring all sessions is unlikely case
    let mut deleted_sessions: Vec<Uuid> = Vec::with_capacity(ses_map.read().unwrap().len());
    let now = Utc::now().naive_utc();

    // Delete sessions from cache while generating a list for later use
    let ses_map = &mut *ses_map.write().unwrap();
    ses_map.retain(|k, v| {
        let not_expired = v.expires_at.gt(&now);
        if !not_expired {
            deleted_sessions.push(*k);
        }
        not_expired
    });

    // Delete sessions from DB
    let result = sessions::Entity::delete_many()
        .filter({
            let mut cond = Condition::any();
            for v in &deleted_sessions {
                cond = cond.add(sessions::Column::Id.eq(v.to_string()))
            }
            cond
        })
        .exec(db)
        .await?;

    // Sanity Check
    let rows_affected: usize = result.rows_affected.try_into().map_err(|_| {
        DbErr::Custom("task_expire_sessions: result.rows_affected.try_into()".to_owned())
    })?;
    if rows_affected != deleted_sessions.len() {
        log::error!("task_expire_sessions: rows_affected != deleted_sessions.len()");
    }

    Ok((rows_affected, deleted_sessions))
}

#[get("/task/expire_sessions")]
pub async fn view_task_expire_sessions(my: web::Data<MainData<'_>>) -> impl Responder {
    match task_expire_sessions(&my.pool, &my.cache.sessions).await {
        Ok((rows_affected, deleted_sessions)) => {
            let body = format!(
                "Sessions Deleted: {:?}\nDB Rows Updated: {:?}",
                deleted_sessions.len(),
                rows_affected
            );
            HttpResponse::Ok()
                .content_type(mime::TEXT_PLAIN_UTF_8)
                .body(body)
        }
        Err(e) => {
            log::error!("view_task_expire_sessions: {}", e);
            let body = "ERROR: view_task_expire_sessions";
            HttpResponse::InternalServerError()
                .content_type(mime::TEXT_PLAIN_UTF_8)
                .body(body)
        }
    }
}

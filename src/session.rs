use crate::init::get_db_pool;
use crate::orm;
use crate::orm::sessions::Entity as Sessions;
use crate::orm::{user_names, users};
use crate::user::ClientUser;
use actix_web::{get, HttpResponse, Responder};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use chrono::{NaiveDateTime, Utc};
use once_cell::sync::OnceCell;
use sea_orm::{entity::*, query::*, DatabaseConnection, DbErr};
use std::collections::hash_map;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub type SessionMap = RwLock<HashMap<Uuid, Session>>;

static START_TIME: OnceCell<NaiveDateTime> = OnceCell::new();
static SESSIONS: OnceCell<SessionMap> = OnceCell::new();
static SALT: OnceCell<SaltString> = OnceCell::new();
static ARGON2: OnceCell<Argon2> = OnceCell::new();

#[inline(always)]
pub fn get_argon2() -> &'static Argon2<'static> {
    unsafe { ARGON2.get_unchecked() }
}
#[inline(always)]
pub fn get_sess() -> &'static SessionMap {
    unsafe { SESSIONS.get_unchecked() }
}
#[inline(always)]
pub fn get_start_time() -> &'static NaiveDateTime {
    unsafe { START_TIME.get_unchecked() }
}

/// MUST be called ONCE before using functions in this module
pub fn init() {
    // Init SALT
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
    let salt = SaltString::new(&salt).unwrap();
    SALT.set(salt).expect("failed to set SALT");

    // Init ARGON2
    let salt = SALT.get().unwrap();
    let argon2 = Argon2::new_with_secret(
        salt.as_bytes(),
        argon2::Algorithm::default(),
        argon2::Version::default(),
        argon2::Params::default(),
    )
    .expect("failed to create argon2");
    if ARGON2.set(argon2).is_err() {
        panic!("failed to set ARGON2");
    }

    if START_TIME.set(Utc::now().naive_utc()).is_err() {
        panic!("failed to set START_TIME");
    }

    if SESSIONS.set(RwLock::new(HashMap::new())).is_err() {
        panic!("failed to set SESSIONS");
    }
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

/// Accepts the actix_web Cookies jar and returns a session, if authentication can be found and made.
pub async fn authenticate_by_cookie(cookies: &actix_session::Session) -> Option<(Uuid, Session)> {
    let token = match cookies.get::<String>("token") {
        Ok(Some(token)) => token,
        _ => return None,
    };
    let uuid = match Uuid::parse_str(&token) {
        Ok(uuid) => uuid,
        Err(e) => {
            log::error!("authenticate_by_cookie: parse_str(): {}", e);
            return None;
        }
    };
    authenticate_by_uuid(get_sess(), &uuid)
        .await
        .map(|session| (uuid, session))
}

pub async fn authenticate_client_ctx(cookies: &actix_session::Session) -> Option<ClientUser> {
    let token = match cookies.get::<String>("token") {
        Ok(Some(token)) => token,
        _ => return None,
    };
    let uuid = match Uuid::parse_str(&token) {
        Ok(uuid) => uuid,
        Err(e) => {
            log::error!("authenticate_client_ctx: parse_str(): {}", e);
            return None;
        }
    };
    let result = authenticate_by_uuid(get_sess(), &uuid).await;

    match result {
        Some(session) => users::Entity::find_by_id(session.user_id)
            .select_only()
            .column(users::Column::Id)
            .left_join(user_names::Entity)
            .column(user_names::Column::Name)
            .into_model::<ClientUser>()
            .one(get_db_pool())
            .await
            .unwrap_or(None),
        None => None,
    }
}

/// Accepts a UUID as a string and returns a session, if the UUID can parse and authenticate.
pub async fn authenticate_by_uuid_string(uuid: String) -> Option<(Uuid, Session)> {
    let uuid = match Uuid::parse_str(&uuid) {
        Ok(uuid) => uuid,
        Err(e) => {
            log::error!("authenticate_by_cookie: parse_str(): {}", e);
            return None;
        }
    };
    authenticate_by_uuid(get_sess(), &uuid)
        .await
        .map(|session| (uuid, session))
}

/// Accepts a uuid::Uuid type and returns a session if the token can authenticate.
/// Use get_session() if you do not want to expire sessions.
pub async fn authenticate_by_uuid(ses_map: &SessionMap, uuid: &Uuid) -> Option<Session> {
    let session = ses_map
        .read()
        .unwrap()
        .get(uuid)
        .map(|uuid| uuid.to_owned());

    // check for expiration, removes and returns none if expired
    let now = Utc::now().naive_utc();
    match session {
        Some(session) => match session.expires_at.lt(&now) {
            true => match remove_session(ses_map, *uuid).await {
                Ok(_) => None,
                Err(e) => {
                    log::error!("authenticate_by_uuid: remove_session(): {}", e);
                    None
                }
            },
            false => Some(session),
        },
        None => None,
    }
}

pub async fn new_session(ses_map: &SessionMap, user_id: i32) -> Result<Uuid, DbErr> {
    // TODO make the expiration duration configurable
    // 20 seconds for testing purposes
    let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(20);
    let ses = Session {
        user_id,
        expires_at,
    };
    let mut uuid;
    loop {
        uuid = Uuid::new_v4();
        if let hash_map::Entry::Vacant(e) = ses_map.write().unwrap().entry(uuid) {
            e.insert(ses);
            break;
        }
    }

    let session = orm::sessions::ActiveModel {
        id: Set(uuid.to_string()),
        user_id: Set(user_id),
        expires_at: Set(expires_at),
    };
    Sessions::insert(session).exec(get_db_pool()).await?;

    Ok(uuid)
}

/// use authenticate_by_uuid() instead unless you have a good reason.
pub fn get_session(ses_map: &SessionMap, uuid: &Uuid) -> Option<Session> {
    ses_map
        .read()
        .unwrap()
        .get(uuid)
        .map(|uuid| uuid.to_owned())
}

/// use get_session() instead unless you have a really good reason to talk to the DB
pub async fn get_session_from_db(uuid: &Uuid) -> Result<Option<orm::sessions::Model>, DbErr> {
    Sessions::find_by_id(uuid.to_string())
        .one(get_db_pool())
        .await
}

pub async fn reload_session_cache(ses_map: &SessionMap) -> Result<(), DbErr> {
    let results = Sessions::find().all(get_db_pool()).await?;
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

pub async fn remove_session(ses_map: &SessionMap, uuid: Uuid) -> Result<Option<Session>, DbErr> {
    // testing indicates if you match the function result directly it holds the mutex.
    // using a let, this should unlock immediately.
    let result = ses_map.write().unwrap().remove(&uuid);
    match result {
        Some(_) => {
            log::info!("remove_session: deleting {}", uuid);
            orm::sessions::Entity::delete_many()
                .filter(orm::sessions::Column::Id.eq(uuid.to_string()))
                .exec(get_db_pool())
                .await?;
            Ok(result)
        }
        None => {
            log::error!("remove_session: UUID not found: {}", uuid);
            Ok(None)
        }
    }
}

pub async fn task_expire_sessions(
    db: &DatabaseConnection,
    ses_map: &SessionMap,
) -> Result<(usize, Vec<Uuid>), DbErr> {
    // allocate memory up front with a read lock to minimize write lock
    // this can be reduced since expiring all sessions is unlikely case
    let ses_map_len = ses_map.read().unwrap().len();
    let mut deleted_sessions: Vec<Uuid> = Vec::with_capacity(ses_map_len);
    let now = Utc::now().naive_utc();

    // Delete sessions from cache while generating a list for later use
    // scoped to ensure mutex gets dropped early
    {
        ses_map.write().unwrap().retain(|k, v| {
            let not_expired = v.expires_at.gt(&now);
            if !not_expired {
                deleted_sessions.push(*k);
            }
            not_expired
        });
    }

    let rows_affected = match deleted_sessions.len() {
        0 => 0,
        _ => {
            // Delete sessions from DB
            let result = orm::sessions::Entity::delete_many()
                .filter({
                    let mut cond = Condition::any();
                    for v in &deleted_sessions {
                        cond = cond.add(orm::sessions::Column::Id.eq(v.to_string()))
                    }
                    cond
                })
                .exec(db)
                .await?;

            // Sanity Check
            let rows_affected: usize = result.rows_affected.try_into().map_err(|_| {
                DbErr::Custom("task_expire_sessions: result.rows_affected.try_into()".to_owned())
            })?;
            rows_affected
        }
    };

    if rows_affected != deleted_sessions.len() {
        log::error!("task_expire_sessions: rows_affected != deleted_sessions.len()");
    }

    Ok((rows_affected, deleted_sessions))
}

#[get("/task/expire_sessions")]
pub async fn view_task_expire_sessions() -> impl Responder {
    match task_expire_sessions(get_db_pool(), get_sess()).await {
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

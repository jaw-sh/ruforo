// Interprets the XF2 PHP-serialized session record (xf_session.session_data).
//
// XF itself (XF\Session\DbStorage + XF\App::getVisitorFromSession) only trusts a
// session that is unexpired and whose `passwordDate` matches the user's current
// xf_user_profile.password_date. We mirror those checks here and read `userId`
// and `passwordDate` only from the top level of the serialized array, instead of
// pattern-matching anywhere inside the blob (where user-influenced strings live).

use super::orm::session;
use super::orm::user;
use super::orm::user_ignored;
use ruforo::web::chat::implement;
use sea_orm::entity::prelude::*;
use sea_orm::{DatabaseConnection, DbBackend, FromQueryResult, QuerySelect, Statement};
use std::time::{SystemTime, UNIX_EPOCH};

/// Top-level integer fields we care about from a PHP-serialized XF session array.
#[derive(Debug, Default, PartialEq, Eq)]
struct XfSessionFields {
    user_id: Option<u64>,
    password_date: Option<u64>,
}

/// Minimal cursor over PHP `serialize()` output. Only what XF session data needs:
/// arrays, strings, ints, floats, bools and null. Objects/references abort parsing.
struct PhpCursor<'a> {
    buf: &'a [u8],
    pos: usize,
}

enum PhpScalar<'a> {
    Int(i64),
    Str(&'a [u8]),
    Other,
}

impl<'a> PhpCursor<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn expect(&mut self, b: u8) -> Option<()> {
        if *self.buf.get(self.pos)? == b {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    /// Reads bytes up to (not including) `delim`, consuming the delimiter.
    fn read_until(&mut self, delim: u8) -> Option<&'a [u8]> {
        let start = self.pos;
        let rel = self.buf.get(start..)?.iter().position(|&c| c == delim)?;
        self.pos = start + rel + 1;
        Some(&self.buf[start..start + rel])
    }

    fn read_int(&mut self, delim: u8) -> Option<i64> {
        std::str::from_utf8(self.read_until(delim)?).ok()?.parse().ok()
    }

    /// Parses one value. Arrays are walked (and discarded) so the cursor stays aligned.
    fn value(&mut self, depth: usize) -> Option<PhpScalar<'a>> {
        if depth > 32 {
            return None;
        }
        let tag = *self.buf.get(self.pos)?;
        self.pos += 1;
        match tag {
            b'N' => {
                self.expect(b';')?;
                Some(PhpScalar::Other)
            }
            b'b' | b'd' => {
                self.expect(b':')?;
                self.read_until(b';')?;
                Some(PhpScalar::Other)
            }
            b'i' => {
                self.expect(b':')?;
                Some(PhpScalar::Int(self.read_int(b';')?))
            }
            b's' => {
                self.expect(b':')?;
                let len = usize::try_from(self.read_int(b':')?).ok()?;
                self.expect(b'"')?;
                let end = self.pos.checked_add(len)?;
                let s = self.buf.get(self.pos..end)?;
                self.pos = end;
                self.expect(b'"')?;
                self.expect(b';')?;
                Some(PhpScalar::Str(s))
            }
            b'a' => {
                self.expect(b':')?;
                let count = self.read_int(b':')?;
                self.expect(b'{')?;
                for _ in 0..count {
                    self.value(depth + 1)?; // key
                    self.value(depth + 1)?; // value
                }
                self.expect(b'}')?;
                Some(PhpScalar::Other)
            }
            // Objects, references, custom serialization: not expected in XF sessions.
            _ => None,
        }
    }
}

/// Extracts top-level `userId` / `passwordDate` from XF session data.
/// Returns None if the blob is not a well-formed top-level PHP array.
fn parse_xf_session(data: &[u8]) -> Option<XfSessionFields> {
    let mut cur = PhpCursor::new(data);
    cur.expect(b'a')?;
    cur.expect(b':')?;
    let count = cur.read_int(b':')?;
    cur.expect(b'{')?;

    let mut fields = XfSessionFields::default();
    for _ in 0..count {
        let key = cur.value(1)?;
        let val = cur.value(1)?;
        if let (PhpScalar::Str(k), PhpScalar::Int(v)) = (key, val) {
            match k {
                b"userId" => fields.user_id = u64::try_from(v).ok(),
                b"passwordDate" => fields.password_date = u64::try_from(v).ok(),
                _ => {}
            }
        }
    }
    cur.expect(b'}')?;

    Some(fields)
}

#[derive(FromQueryResult)]
struct XfPasswordDate {
    password_date: u32,
}

async fn get_password_date(db: &DatabaseConnection, user_id: u32) -> Option<u32> {
    match XfPasswordDate::find_by_statement(Statement::from_sql_and_values(
        DbBackend::MySql,
        "SELECT password_date FROM xf_user_profile WHERE user_id = ?",
        vec![user_id.into()],
    ))
    .one(db)
    .await
    {
        Ok(row) => row.map(|r| r.password_date),
        Err(err) => {
            log::warn!("Failed to fetch XF password date: {:?}", err);
            None
        }
    }
}

#[derive(FromQueryResult)]
struct XfSession {
    pub id: u32,
    pub username: String,
    pub avatar_date: u32,
    pub avatar_format: Option<String>,
    pub is_staff: bool,
}

pub fn avatar_uri(id: u32, date: u32, format: Option<&str>) -> String {
    if date > 0 {
        let ext = format.filter(|s| !s.is_empty()).unwrap_or("jpg");
        format!(
            "{}/data/avatars/m/{}/{}.{}?{}",
            std::env::var("XF_PUBLIC_URL").expect("XF_PUBLIC_URL must be set in .env"),
            id / 1000,
            id,
            ext,
            date
        )
    } else {
        String::new()
    }
}

impl Default for XfSession {
    fn default() -> Self {
        Self {
            id: 0,
            username: "Guest".to_owned(),
            avatar_date: 0,
            avatar_format: None,
            is_staff: false,
        }
    }
}

pub async fn get_user_id_from_cookie(db: &DatabaseConnection, cookie: &String) -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now = u32::try_from(now).unwrap_or(u32::MAX);

    // Same predicate as XF\Session\DbStorage::getSession(): expired rows are dead.
    let session = match session::Entity::find_by_id(cookie.as_bytes().to_vec())
        .filter(session::Column::ExpiryDate.gte(now))
        .one(db)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => return 0,
        Err(err) => {
            log::warn!("Failed to fetch user session: {:?}", err);
            return 0;
        }
    };

    let fields = match parse_xf_session(&session.session_data) {
        Some(fields) => fields,
        None => {
            log::debug!("Unparseable XF session data; treating as guest.");
            return 0;
        }
    };

    let user_id = match fields.user_id.and_then(|id| u32::try_from(id).ok()) {
        Some(id) if id > 0 => id,
        _ => return 0,
    };

    // Mirror XF\App::getVisitorFromSession(): a password change invalidates sessions.
    let session_pw_date = fields.password_date.unwrap_or(0);
    let user_pw_date = get_password_date(db, user_id).await.unwrap_or(0);
    if session_pw_date != u64::from(user_pw_date) {
        log::debug!("Session passwordDate mismatch for user {}; treating as guest.", user_id);
        return 0;
    }

    log::debug!("User {:?} has authorized.", user_id);
    user_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_top_level_user_id() {
        let data = br#"a:3:{s:6:"userId";i:42;s:12:"passwordDate";i:1700000000;s:16:"dismissedNotices";a:0:{}}"#;
        assert_eq!(
            parse_xf_session(data),
            Some(XfSessionFields { user_id: Some(42), password_date: Some(1700000000) })
        );
    }

    #[test]
    fn ignores_nested_and_string_embedded_user_id() {
        // A user-influenced string containing a fake userId must not be trusted,
        // nor may a nested array's userId key.
        let data = br#"a:2:{s:8:"redirect";s:21:"s:6:"userId";i:1;xxxx";s:5:"inner";a:1:{s:6:"userId";i:1;}}"#;
        assert_eq!(parse_xf_session(data), Some(XfSessionFields::default()));
    }

    #[test]
    fn rejects_malformed() {
        assert_eq!(parse_xf_session(b"s:6:\"userId\";i:1;"), None);
        assert_eq!(parse_xf_session(b"a:1:{s:6:\"userId\";i:1;"), None);
        assert_eq!(parse_xf_session(b"a:1:{s:99:\"userId\";i:1;}"), None);
    }
}

pub async fn get_session_with_user_id(db: &DatabaseConnection, id: u32) -> implement::Session {
    // Fetch basic user info
    let session = if id > 0 {
        match user::Entity::find_by_id(id)
            .select_only()
            .column_as(user::Column::UserId, "id")
            .column(user::Column::Username)
            .column(user::Column::AvatarDate)
            .column(user::Column::AvatarFormat)
            .column(user::Column::IsStaff)
            .filter(user::Column::UserId.eq(id))
            .filter(user::Column::UserState.eq("valid"))
            .filter(user::Column::IsBanned.eq(false))
            .into_model::<XfSession>()
            .one(db)
            .await
        {
            Ok(res) => match res {
                Some(session) => session,
                None => {
                    log::debug!("No result for user id {:?} (banned / invalid?)", id);
                    XfSession::default()
                }
            },
            Err(err) => {
                log::warn!("MySQL Error: {:?}", err);
                XfSession::default()
            }
        }
    } else {
        XfSession::default()
    };

    // Fetch additional information
    let ignored_users: Vec<u32> = if session.id > 0 {
        match user_ignored::Entity::find()
            .filter(user_ignored::Column::UserId.eq(id))
            .all(db)
            .await
        {
            Ok(res) => res
                .into_iter()
                .map(|m| m.ignored_user_id)
                .collect::<Vec<u32>>(),
            Err(err) => {
                log::warn!("MySQL Error: {:?}", err);
                Default::default()
            }
        }
    } else {
        Default::default()
    };

    implement::Session {
        id: session.id,
        username: session.username,
        avatar_url: avatar_uri(session.id, session.avatar_date, session.avatar_format.as_deref()),
        ignored_users,
        is_staff: session.is_staff,
    }
}

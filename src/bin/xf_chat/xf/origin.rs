// Discovers the browser origins that serve the chat, from the XF MultiSite
// (HappyBoard/MultiSite) domain tables, so the /chat.ws Origin allow-list
// covers every mirror domain without hand-maintained configuration.

use ruforo::web::chat::{host_to_origin, set_dynamic_allowed_origins, url_origin};
use sea_orm::{DatabaseConnection, DbBackend, FromQueryResult, Statement};
use std::time::Duration;

/// How often the domain list is re-read from the database.
const REFRESH_INTERVAL: Duration = Duration::from_secs(300);

#[derive(FromQueryResult)]
struct HostRow {
    host: String,
    home_url: String,
}

async fn load_origins(db: &DatabaseConnection) -> Result<Vec<String>, sea_orm::DbErr> {
    // Active domains (host + home_url) and active aliases of active domains.
    let rows = HostRow::find_by_statement(Statement::from_string(
        DbBackend::MySql,
        "SELECT d.host AS host, d.home_url AS home_url
           FROM xf_hb_ms_domain d
          WHERE d.active = 1
         UNION ALL
         SELECT a.host AS host, '' AS home_url
           FROM xf_hb_ms_domain_alias a
           JOIN xf_hb_ms_domain d ON d.domain_id = a.domain_id
          WHERE a.active = 1 AND d.active = 1"
            .to_owned(),
    ))
    .all(db)
    .await?;

    let mut origins: Vec<String> = Vec::new();
    for row in rows {
        for origin in [host_to_origin(&row.host), url_origin(&row.home_url)] {
            if !origin.is_empty() && origin.contains("://") && !origins.contains(&origin) {
                origins.push(origin);
            }
        }
    }
    Ok(origins)
}

async fn refresh(db: &DatabaseConnection) {
    match load_origins(db).await {
        Ok(origins) => {
            log::info!("Chat WebSocket allowed MultiSite origins: {:?}", origins);
            set_dynamic_allowed_origins(origins);
        }
        // Table missing (MultiSite not installed) or DB hiccup: keep the previous list.
        Err(err) => log::warn!("Unable to load MultiSite chat origins: {:?}", err),
    }
}

/// Loads the origin list once, then keeps refreshing it in the background.
pub async fn start_origin_refresher(db: DatabaseConnection) {
    refresh(&db).await;
    actix_web::rt::spawn(async move {
        let mut interval = actix_web::rt::time::interval(REFRESH_INTERVAL);
        interval.tick().await; // first tick fires immediately; already loaded above
        loop {
            interval.tick().await;
            refresh(&db).await;
        }
    });
}

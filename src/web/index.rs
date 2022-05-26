use crate::middleware::ClientCtx;
use actix_web::{get, Error, Responder};

pub(super) fn configure(conf: &mut actix_web::web::ServiceConfig) {
    conf.service(view_index);
}

#[get("/")]
async fn view_index(client: ClientCtx) -> Result<impl Responder, Error> {
    // In XenForo and most forums, the default behavior is to render the forum index.
    // However this is usually an option and sometimes forums are under /forums/.
    super::forum::render_forum_list(client).await
}

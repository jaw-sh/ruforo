use crate::middleware::ClientCtx;
use actix_web::{get, Error, Responder};

#[get("/")]
async fn view_index(client: ClientCtx) -> Result<impl Responder, Error> {
    // In XenForo and most forums, the default behavior is to render the forum index.
    // However this is usually an option and sometimes forums are under /forums/.
    crate::forum::render_forum_list(client).await
}

#[get("/permission-test")]
async fn view_permission_test(client: ClientCtx) -> Result<impl Responder, Error> {
    crate::forum::render_forum_list(client).await
}

mod append_context;
mod identity;

pub use append_context::AppendContext;
pub use identity::CookieIdentityPolicy as IdentityPolicy;

// Documentation for middleware can be found here:
// https://github.com/actix/actix-web/blob/master/src/middleware/normalize.rs

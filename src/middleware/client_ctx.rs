use crate::get_db_pool;
use crate::permission::PermissionData;
use crate::user::ClientUser;
use actix_session::Session;
use actix_utils::future::{ok, Ready};
use actix_web::dev::{
    forward_ready, Extensions, Payload, Service, ServiceRequest, ServiceResponse, Transform,
};
use actix_web::{web::Data, Error, FromRequest, HttpMessage, HttpRequest};
use futures_util::future::{FutureExt as _, LocalBoxFuture};
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Client data stored for a single request cycle.
/// Distinct from ClientCtx because it is defined through request data.
#[derive(Clone, Debug)]
pub struct ClientCtxInner {
    pub client: Option<ClientUser>,
    pub groups: Vec<i32>,
    pub permissions: Option<Arc<PermissionData>>,
    pub request_start: Instant,
}

impl ClientCtxInner {
    fn new() -> Self {
        Self {
            client: None,
            groups: Vec::new(),
            permissions: None,
            request_start: Instant::now(),
        }
    }
}

/// Client context passed to routes.
/// Wraps ClientCtxInner, which is set at the beginning of the request.
#[derive(Clone, Debug)]
pub struct ClientCtx(Rc<RefCell<ClientCtxInner>>);

impl ClientCtx {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ClientCtxInner::new())))
    }

    fn get_client_ctx(extensions: &mut Extensions, permissions: &Arc<PermissionData>) -> Self {
        let ctx = match extensions.get::<Rc<RefCell<ClientCtxInner>>>() {
            // Existing record in extensions; pull it.
            Some(s_impl) => Self(Rc::clone(s_impl)),
            // No existing record; create and insert it.
            None => {
                let inner = Rc::new(RefCell::new(ClientCtxInner::new()));
                extensions.insert(inner.clone());
                Self(inner)
            }
        };
        // Add permission Arc reference to our inner value.
        ctx.0.borrow_mut().permissions = Some(permissions.clone());
        ctx
    }

    pub fn get_groups(&self) -> Vec<i32> {
        self.0.borrow().groups.to_owned()
    }

    /// Returns either the user's id or None.
    pub fn get_id(&self) -> Option<i32> {
        self.0.borrow().client.as_ref().map(|u| u.id)
    }

    /// Returns either the user's name or the word for guest.
    /// TODO: l10n "Guest"
    pub fn get_name(&self) -> String {
        let user = &self.0.borrow().client;
        match user {
            Some(user) => user.name.to_owned(),
            None => "Guest".to_owned(),
        }
    }

    pub fn is_user(&self) -> bool {
        self.0.borrow().client.is_some()
    }

    pub fn can(&self, tag: &str) -> bool {
        let inner = self.0.borrow();
        match &inner.permissions {
            // Permission data present, evaluate
            Some(permissions) => permissions.can(&self, tag),
            // Permission data not present?
            None => {
                log::warn!(
                    "Bad client permission check for {:?} - no permission data!?",
                    &inner.client
                );
                false
            }
        }
    }

    pub fn can_post_in_thread(&self, _thread: &crate::orm::threads::Model) -> bool {
        self.can_post_in_forum()
    }

    pub fn can_post_in_forum(&self) -> bool {
        true
    }

    pub fn can_delete_post(&self, post: &crate::post::PostForTemplate) -> bool {
        self.is_user() && self.get_id() == post.user_id
    }

    pub fn can_update_post(&self, post: &crate::post::PostForTemplate) -> bool {
        self.is_user() && self.get_id() == post.user_id
    }

    pub fn can_read_post(&self, post: &crate::post::PostForTemplate) -> bool {
        // TODO: In XenForo, users cannot view their own deleted posts.
        // This should be a moderator setting. Maybe a 'can view own deleted posts' option.
        post.deleted_at.is_none() || self.get_id() == post.user_id
    }

    /// Returns Duration representing request time.
    pub fn request_time(&self) -> Duration {
        Instant::now() - self.0.borrow().request_start
    }

    /// Returns human readable representing request time.
    pub fn request_time_as_string(&self) -> String {
        let us = self.request_time().as_micros();
        if us > 5000 {
            format!("{}ms", us / 1000)
        } else {
            format!("{}μs", us)
        }
    }
}

/// This implementation is what actually provides the `client: ClientCtx` in the parameters of route functions.
impl FromRequest for ClientCtx {
    /// The associated error which can be returned.
    type Error = Error;
    /// Future that resolves to a Self.
    type Future = Ready<Result<Self, Self::Error>>;

    /// Create a Self from request parts asynchronously.
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let perm_arc = req
            .app_data::<Data<Arc<PermissionData>>>()
            .expect("No PermissionData in FromRequest.");
        ok(ClientCtx::get_client_ctx(
            &mut req.extensions_mut(),
            perm_arc,
        ))
    }
}

impl<S, B> Transform<S, ServiceRequest> for ClientCtx
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ClientCtxMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ClientCtxMiddleware {
            service,
            inner: self.0.clone(),
        })
    }
}

/// Client context middleware
pub struct ClientCtxMiddleware<S> {
    service: S,
    #[allow(dead_code)]
    inner: Rc<RefCell<ClientCtxInner>>,
}

impl<S, B> Service<ServiceRequest> for ClientCtxMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Borrows of `req` must be done in a precise way to avoid conflcits. This order is important.
        let (httpreq, payload) = req.into_parts();
        let cookies = Session::extract(&httpreq).into_inner();
        let req = ServiceRequest::from_parts(httpreq, payload);
        //let db = req
        //    .app_data::<&'static sea_orm::DatabaseConnection>()
        //    .expect("No database connection available through web server.");
        let perm_arc = req
            .app_data::<Data<Arc<PermissionData>>>()
            .expect("No permission data available through web server.");
        let ctx = ClientCtx::get_client_ctx(&mut *req.extensions_mut(), perm_arc);
        let fut = self.service.call(req);

        async move {
            use crate::group::get_group_ids_for_client;
            use crate::session::authenticate_client_by_session;

            match cookies {
                Ok(cookies) => {
                    let mut inner = ctx.0.borrow_mut();

                    // Assign the user to our ClientCtx struct.
                    inner.client = authenticate_client_by_session(&cookies).await;

                    // Add permission groups used by this connection.
                    inner.groups = get_group_ids_for_client(get_db_pool(), &inner.client).await;
                }
                Err(e) => {
                    log::error!("ClientCtxMiddleware: Session::extract(): {}", e);
                }
            };
            Ok(fut.await?)
        }
        .boxed_local()
    }
}

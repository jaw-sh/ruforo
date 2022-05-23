use crate::permission::PermissionData;
use crate::session::authenticate_client_ctx;
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
    pub permission: Option<Arc<PermissionData>>,
    pub request_start: Instant,
}

impl ClientCtxInner {
    fn new() -> Self {
        Self {
            client: None,
            permission: None,
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
        ctx.0.borrow_mut().permission = Some(permissions.clone());
        ctx
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
        true
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
        let (httpreq, payload) = req.into_parts();
        let cookies = Session::extract(&httpreq).into_inner();
        let req = ServiceRequest::from_parts(httpreq, payload);
        let perm_arc = req
            .app_data::<Data<Arc<PermissionData>>>()
            .expect("No permission data available.");
        let ctx = ClientCtx::get_client_ctx(&mut *req.extensions_mut(), perm_arc);
        let fut = self.service.call(req);

        async move {
            match cookies {
                Ok(cookies) => {
                    let result = authenticate_client_ctx(&cookies).await;

                    // Assign the user to our ClientCtx struct.
                    if let Some(user) = result {
                        ctx.0.borrow_mut().client = Some(user);
                    }
                }
                Err(e) => {
                    log::error!("ClientCtxMiddleware: Session::extract(): {}", e);
                }
            };
            let result = fut.await?;
            Ok(result)
        }
        .boxed_local()
    }
}

use crate::session::authenticate_by_cookie;
use crate::user::{Client, ClientUser};
use actix_session::Session;
use actix_utils::future::{ok, Ready};
use actix_web::{
    dev::{
        forward_ready, Extensions, Payload, Service, ServiceRequest, ServiceResponse, Transform,
    },
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{FutureExt as _, LocalBoxFuture};
use std::time::{Duration, Instant};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct ClientCtx(Rc<RefCell<ClientInner>>);

#[derive(Clone, Debug)]
pub struct ClientInner {
    pub request_start: Instant,
    pub client: Client,
}

impl ClientInner {
    fn new() -> Self {
        Self {
            request_start: Instant::now(),
            client: Client::new(),
        }
    }
}

impl ClientCtx {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(ClientInner::new())))
    }

    fn get_client_ctx(extensions: &mut Extensions) -> Self {
        if let Some(s_impl) = extensions.get::<Rc<RefCell<ClientInner>>>() {
            return Self(Rc::clone(s_impl));
        }
        let inner = Rc::new(RefCell::new(ClientInner::new()));
        extensions.insert(inner.clone());
        Self(inner)
    }

    /// Returns either the user's id or None.
    pub fn get_id(&self) -> Option<i32> {
        self.0.borrow().client.user.as_ref().map(|u| u.id)
    }

    /// Returns either the user's name or the word for guest.
    /// TODO: l10n "Guest"
    pub fn get_name(&self) -> String {
        let user = &self.0.borrow().client.user;
        match user {
            Some(user) => user.name.to_owned(),
            None => "Guest".to_owned(),
        }
    }

    pub fn is_user(&self) -> bool {
        self.0.borrow().client.user.is_some()
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

    pub fn can_read_post(&self, _post: &crate::post::PostForTemplate) -> bool {
        true
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

pub trait ClientSession {
    fn get_client_ctx(&self) -> ClientCtx;
}

impl ClientSession for HttpRequest {
    fn get_client_ctx(&self) -> ClientCtx {
        ClientCtx::get_client_ctx(&mut *self.extensions_mut())
    }
}

impl ClientSession for ServiceRequest {
    fn get_client_ctx(&self) -> ClientCtx {
        ClientCtx::get_client_ctx(&mut *self.extensions_mut())
    }
}

impl FromRequest for ClientCtx {
    /// The associated error which can be returned.
    type Error = Error;
    /// Future that resolves to a Self.
    type Future = Ready<Result<Self, Self::Error>>;

    /// Create a Self from request parts asynchronously.
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ok(ClientCtx::get_client_ctx(&mut req.extensions_mut()))
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

pub struct ClientCtxMiddleware<S> {
    service: S,
    inner: Rc<RefCell<ClientInner>>,
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
        let ctx = ClientCtx::get_client_ctx(&mut *req.extensions_mut());
        let fut = self.service.call(req);
        async move {
            match cookies {
                Ok(cookies) => {
                    let result = authenticate_by_cookie(&cookies).await;
                    match result {
                        Some((uuid, session)) => {
                            let x = ctx.0.borrow_mut().client.user = Some(ClientUser {
                                id: session.user_id,
                                name: "TMP FIXME".to_owned(),
                            });
                        }
                        None => {}
                    };
                }
                Err(_) => {
                }
            };
            Ok(fut.await?)
        }
        .boxed_local()
    }
}

use crate::db::get_db_pool;
use crate::permission::PermissionData;
use crate::user::Profile;
use actix::fut::ready;
use actix_session::Session;
use actix_web::dev::{
    self, Extensions, Payload, Service, ServiceRequest, ServiceResponse, Transform,
};
use actix_web::{web::Data, Error, FromRequest, HttpMessage, HttpRequest};
use futures::future::{err, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Client data stored for a single request cycle.
/// Distinct from ClientCtx because it is defined through request data.
#[derive(Clone, Debug)]
pub struct ClientCtxInner {
    /// User data. Optional. None is a guest user.
    pub client: Option<Profile>,
    /// List of user group ids. Guests may receive unregistered/portal roles.
    pub groups: Vec<i32>,
    /// Permission data.
    pub permissions: Data<PermissionData>,
    /// Randomly generated string for CSR.
    pub nonce: String,
    /// Time the request started for page load statistics.
    pub request_start: Instant,
}

impl Default for ClientCtxInner {
    fn default() -> Self {
        Self {
            // Guests and users.
            permissions: Data::new(PermissionData::default()),
            groups: Vec::new(),
            // Only users.
            client: None,
            // Generally left default.
            nonce: Self::nonce(),
            request_start: Instant::now(),
        }
    }
}

impl ClientCtxInner {
    pub async fn from_session(session: &Session, permissions: Data<PermissionData>) -> Self {
        use crate::group::get_group_ids_for_client;
        use crate::session::authenticate_client_by_session;

        let db = get_db_pool();
        let client = authenticate_client_by_session(session).await;
        let groups = get_group_ids_for_client(db, &client).await;

        ClientCtxInner {
            client,
            groups,
            permissions,
            ..Default::default()
        }
    }

    /// Returns a hash unique to each request used for CSP.
    /// See: <https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/nonce>
    /// and <https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP>
    pub fn nonce() -> String {
        let mut hasher = blake3::Hasher::new();

        // Hash: Salt
        match std::env::var("SALT") {
            Ok(v) => hasher.update(v.as_bytes()),
            Err(_) => hasher.update("NO_SALT_FOR_NONCE".as_bytes()),
        };

        // Hash: Timestamp
        use std::time::{SystemTime, UNIX_EPOCH};
        hasher.update(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System clock before 1970. Really?")
                .as_millis()
                .to_ne_bytes(),
        );
        hasher.finalize().to_string()
    }
}

/// Client context passed to routes.
/// Wraps ClientCtxInner, which is set at the beginning of the request.
#[derive(Clone, Debug)]
pub struct ClientCtx(Data<ClientCtxInner>);

impl Default for ClientCtx {
    fn default() -> Self {
        Self(Data::new(ClientCtxInner::default()))
    }
}

impl ClientCtx {
    /// Returns instance of Self with components required for ClientCtxInner.
    pub async fn from_session(session: &Session, permissions: Data<PermissionData>) -> Self {
        Self(Data::new(
            ClientCtxInner::from_session(session, permissions).await,
        ))
    }

    pub fn get_or_default_from_extensions(
        extensions: &mut Extensions,
        permissions: Data<PermissionData>,
    ) -> Self {
        match extensions.get::<Data<ClientCtxInner>>() {
            // Existing record in extensions; pull it and return clone.
            Some(cbox) => Self(cbox.clone()),
            // No existing record; create and insert it.
            None => {
                let cbox = Data::new(ClientCtxInner {
                    // Add permission Arc reference to our inner value.
                    permissions: permissions,
                    ..Default::default()
                });
                // Insert ClientCtx into extensions jar.
                extensions.insert(cbox.clone());
                Self(cbox)
            }
        }
    }

    pub fn get_groups(&self) -> Vec<i32> {
        self.0.groups.to_owned()
    }

    /// Returns either the user's id or None.
    pub fn get_id(&self) -> Option<i32> {
        self.0.client.as_ref().map(|u| u.id)
    }

    /// Returns either the user's name or the word for guest.
    /// TODO: l10n "Guest"
    pub fn get_name(&self) -> String {
        let user = &self.0.client;
        match user {
            Some(user) => user.name.to_owned(),
            None => "Guest".to_owned(),
        }
    }

    pub fn get_user(&self) -> Option<&Profile> {
        self.0.client.as_ref()
    }

    pub fn is_user(&self) -> bool {
        self.0.client.is_some()
    }

    pub fn can(&self, tag: &str) -> bool {
        self.0.permissions.can(self, tag)
    }

    pub fn can_post_in_thread(&self, _thread: &crate::orm::threads::Model) -> bool {
        self.can_post_in_forum()
    }

    pub fn can_post_in_forum(&self) -> bool {
        true
    }

    pub fn can_delete_post(&self, post: &crate::web::post::PostForTemplate) -> bool {
        self.is_user() && self.get_id() == post.user_id
    }

    pub fn can_update_post(&self, post: &crate::web::post::PostForTemplate) -> bool {
        self.is_user() && self.get_id() == post.user_id
    }

    pub fn can_read_post(&self, post: &crate::web::post::PostForTemplate) -> bool {
        // TODO: In XenForo, users cannot view their own deleted posts.
        // This should be a moderator setting. Maybe a 'can view own deleted posts' option.
        post.deleted_at.is_none() || self.get_id() == post.user_id
    }

    pub fn get_nonce(&self) -> &String {
        &self.0.nonce
    }

    pub fn get_permissions(&self) -> &Data<PermissionData> {
        &self.0.permissions
    }

    /// Returns Duration representing request time.
    pub fn request_time(&self) -> Duration {
        Instant::now() - self.0.request_start
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
        if let Some(perm_arc) = req.app_data::<Data<PermissionData>>() {
            ready(Ok(ClientCtx::get_or_default_from_extensions(
                &mut req.extensions_mut(),
                perm_arc.clone(),
            )))
        } else {
            err(actix_web::error::ErrorServiceUnavailable(
                "Permission data is not loaded.",
            ))
        }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for ClientCtx
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ClientCtxMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ClientCtxMiddleware {
            service: Rc::new(service),
            inner: self.0.clone(),
        }))
    }
}

/// Client context middleware
pub struct ClientCtxMiddleware<S> {
    service: Rc<S>,
    #[allow(dead_code)]
    inner: Data<ClientCtxInner>,
}

impl<S, B> Service<ServiceRequest> for ClientCtxMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        // Borrows of `req` must be done in a precise way to avoid conflcits. This order is important.
        let (httpreq, payload) = req.into_parts();
        let session = Session::extract(&httpreq).into_inner();
        let req = ServiceRequest::from_parts(httpreq, payload);

        // If we do not have permission data there is no client interface to access.
        Box::pin(async move {
            if let Some(perm_arc) = req.app_data::<Data<PermissionData>>() {
                let perm_arc = perm_arc.clone();

                match session {
                    Ok(session) => req.extensions_mut().insert(Data::new(
                        ClientCtxInner::from_session(&session, perm_arc).await,
                    )),
                    Err(err) => {
                        log::error!("Unable to extract Session data in middleware: {}", err);
                        None
                    }
                };
            };

            svc.call(req).await
        })
    }
}

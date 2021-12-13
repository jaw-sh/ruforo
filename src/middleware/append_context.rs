use crate::frontend::Context;
use crate::session::MainData;
use crate::user::{get_client_from_identity, Client};
use actix_identity::Identity;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, FromRequest, HttpMessage,
};
use std::future::{ready, Ready};
use std::time::Instant;

#[derive(Debug, Clone, Copy, Default)]
pub struct AppendContext {}

impl<S, B> Transform<S, ServiceRequest> for AppendContext
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AppendContextMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AppendContextMiddleware { service }))
    }
}

pub struct AppendContextMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AppendContextMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // get mut HttpRequest from ServiceRequest
        let (httpreq, _payload) = &req.parts_mut();

        // insert data into extensions if enabled
        httpreq.extensions_mut().insert(Context {
            request_start: Instant::now(),
        });

        // log in using cookies
        let client = match Identity::extract(httpreq).into_inner() {
            Ok(id) => match httpreq.app_data::<Data<MainData>>() {
                Some(data) => futures::executor::block_on(get_client_from_identity(&data, &id)),
                None => Client::default(),
            },
            Err(_) => Client::default(),
        };

        httpreq.extensions_mut().insert(client);
        self.service.call(req)
    }
}

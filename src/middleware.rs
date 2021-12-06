use crate::frontend::Context;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use chrono::prelude::Utc;
use std::future::{ready, Ready};

// Documentation for middleware can be found here:
// https://github.com/actix/actix-web/blob/master/src/middleware/normalize.rs

#[derive(Debug, Clone, Copy)]
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
        println!("request is passing through the AppendContext middleware");

        // get mut HttpRequest from ServiceRequest
        let (httpreq, _payload) = req.parts_mut();

        // insert data into extensions if enabled
        httpreq.extensions_mut().insert(Context {
            request_start: Utc::now().naive_utc(),
        });

        self.service.call(req)
    }
}

use crate::frontend::Context;
use crate::session::MainData;
use crate::user::Client;
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
        use crate::orm::users::Entity as User;
        use actix_identity::Identity;
        use sea_orm::entity::*;

        // get mut HttpRequest from ServiceRequest
        let (httpreq, _payload) = &req.parts_mut();
        println!("1. Starting auth.");

        // insert data into extensions if enabled
        let context = Context {
            client: match Identity::extract(&httpreq).into_inner() {
                Ok(id) => match httpreq.app_data::<Data<MainData>>() {
                    Some(data) => Client {
                        user: match id.identity() {
                            Some(id) => match crate::session::authenticate_by_uuid_string(
                                &data.cache.sessions,
                                id,
                            ) {
                                Some(session) => futures::executor::block_on(async move {
                                    println!("AUTHED AS USER #{}", session.session.user_id);
                                    User::find_by_id(session.session.user_id)
                                        .one(&data.pool)
                                        .await
                                        .unwrap_or(None)
                                }),
                                None => None,
                            },
                            None => None,
                        },
                    },
                    None => Client::default(),
                },
                Err(_) => Client::default(),
            },
            request_start: Instant::now(),
        };

        futures::executor::block_on(async move {
            println!("2b. Sneed");
        });

        println!("3. Signed in as: {}", context.client.get_name());
        httpreq.extensions_mut().insert(context);
        self.service.call(req)
    }
}

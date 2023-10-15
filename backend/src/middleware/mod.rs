pub mod roles;
pub mod token;
use actix_web::{
    dev::{self, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web::{self},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{self, err, ok, LocalBoxFuture};
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::StateManager;

use self::token::extract_access_token;

#[derive(Debug, Clone)]
pub struct YakManPrinciple {
    pub user_uuid: Option<String>,
}

impl FromRequest for YakManPrinciple {
    type Error = actix_web::Error;
    type Future = future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<YakManPrinciple>() {
            Some(principle) => return ok(principle.clone()),
            None => return err(actix_web::error::ErrorBadRequest("Access Denied")), // todo: fix/clean up
        };
    }
}

pub struct YakManPrincipleTransformer;

impl<S: 'static, B> Transform<S, ServiceRequest> for YakManPrincipleTransformer
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = YakManPrincipleMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(YakManPrincipleMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct YakManPrincipleMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for YakManPrincipleMiddleware<S>
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

        Box::pin(async move {
            let mut user_uuid: Option<String> = None;
            let state = req.app_data::<web::Data<StateManager>>().unwrap();
            if let Some(token) = extract_access_token(&req) {
                match state.get_token_service().validate_access_token(&token) {
                    Ok(claims) => {
                        let uuid = claims.uuid;
                        user_uuid = Some(uuid);
                    }
                    Err(_) => (),
                }
            }

            req.extensions_mut().insert(YakManPrinciple {
                user_uuid: user_uuid,
            });

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

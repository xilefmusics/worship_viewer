use std::future::{Ready, ready};
use std::rc::Rc;

use actix_cors::Cors;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web::Data;
use actix_web::{
    Error, HttpMessage,
    http::{Method, header},
};
use futures_util::future::LocalBoxFuture;

use super::authorization_bearer;
use crate::database::Database;
use crate::error::AppError;
use crate::resources::{SessionModel, User, UserRole};
use crate::settings::Settings;

#[derive(Clone, Default)]
pub struct RequireUser;

impl<S, B> Transform<S, ServiceRequest> for RequireUser
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireUserMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireUserMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireUserMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireUserMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let db = req
            .app_data::<Data<Database>>()
            .cloned()
            .ok_or_else(|| AppError::Internal("database handle missing".into()))
            .map_err(Error::from);

        let cookie_name = Settings::global().cookie_name.clone();
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let db = match db {
                Ok(data) => data,
                Err(err) => return Err(err),
            };

            let session_id = authorization_bearer(&req)
                .or_else(|| {
                    req.cookie(&cookie_name)
                        .map(|cookie| cookie.value().to_owned())
                })
                .ok_or_else(|| AppError::unauthorized())?;

            let user = match db
                .get_session_and_update_user_metrics_or_delete_if_exipired(&session_id)
                .await
            {
                Ok(Some(session)) => session.user,
                Ok(None) => return Err(AppError::unauthorized().into()),
                Err(err) => return Err(err.into()),
            };

            req.extensions_mut().insert(user);

            let response = service.call(req).await?;
            Ok(response)
        })
    }
}

#[derive(Clone, Default)]
pub struct RequireAdmin;

impl<S, B> Transform<S, ServiceRequest> for RequireAdmin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAdminMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireAdminMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequireAdminMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequireAdminMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let is_admin = req
                .extensions()
                .get::<User>()
                .map(|user| user.role == UserRole::Admin)
                .unwrap_or(false);

            if !is_admin {
                return Err(AppError::forbidden().into());
            }

            service.call(req).await
        })
    }
}

pub fn cors() -> Cors {
    let settings = Settings::global();
    let origin = settings.frontend_origin.clone();
    Cors::default()
        .allowed_origin(&origin)
        .allowed_methods(vec![Method::GET, Method::POST, Method::OPTIONS])
        .allowed_headers(vec![
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
        ])
        .supports_credentials()
}

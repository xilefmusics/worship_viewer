use std::future::{Ready, ready};
use std::rc::Rc;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;

use super::authorization_bearer;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::user::Role as UserRole;
use crate::resources::user::session::service::SessionServiceHandle;
use crate::settings::CookieConfig;

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
        let svc = req
            .app_data::<Data<SessionServiceHandle>>()
            .cloned()
            .ok_or_else(|| AppError::Internal("session service handle missing".into()))
            .map_err(Error::from);

        let cookie_cfg = req
            .app_data::<Data<CookieConfig>>()
            .cloned()
            .ok_or_else(|| AppError::Internal("cookie config missing".into()))
            .map_err(Error::from);
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let svc = match svc {
                Ok(data) => data,
                Err(err) => return Err(err),
            };
            let cookie_cfg = match cookie_cfg {
                Ok(data) => data,
                Err(err) => return Err(err),
            };

            let session_id = authorization_bearer(&req)
                .or_else(|| {
                    req.cookie(&cookie_cfg.name)
                        .map(|cookie| cookie.value().to_owned())
                })
                .ok_or_else(AppError::unauthorized)?;

            let user = match svc.validate_session_and_update_metrics(&session_id).await {
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

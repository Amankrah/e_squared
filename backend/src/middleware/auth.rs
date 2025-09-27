use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use actix_session::{Session, SessionExt};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;
use tracing::warn;

use crate::handlers::AuthService;
use crate::utils::errors::AppError;

pub struct AuthMiddleware {
    auth_service: Rc<AuthService>,
}

impl AuthMiddleware {
    pub fn new(auth_service: AuthService) -> Self {
        Self {
            auth_service: Rc::new(auth_service),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            auth_service: self.auth_service.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    auth_service: Rc<AuthService>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let auth_service = self.auth_service.clone();

        Box::pin(async move {
            // Get session from request extensions (set by SessionMiddleware)
            let session = req.get_session();

            // Try to get token from HTTP-only cookie first
            let token = if let Ok(cookies) = req.cookies() {
                if let Some(cookie) = cookies.iter().find(|c| c.name() == "auth_token") {
                    let token_value = cookie.value().to_string();
                    // Validate that token matches session
                    if let Ok(Some(session_authenticated)) = session.get::<bool>("authenticated") {
                        if !session_authenticated {
                            warn!("Token present but session not authenticated");
                            return Err(AppError::InvalidToken.into());
                        }
                    }
                    token_value
                } else {
                    // Fallback to Authorization header for API requests
                    if let Some(auth_header) = req.headers().get("Authorization") {
                        let header_str = auth_header.to_str().map_err(|_| {
                            warn!("Invalid Authorization header encoding");
                            AppError::InvalidToken
                        })?;
                        if header_str.starts_with("Bearer ") {
                            header_str[7..].to_string()
                        } else {
                            warn!("Authorization header missing Bearer prefix");
                            return Err(AppError::InvalidToken.into());
                        }
                    } else {
                        return Err(AppError::MissingToken.into());
                    }
                }
            } else {
                // Fallback to Authorization header for API requests
                if let Some(auth_header) = req.headers().get("Authorization") {
                    let header_str = auth_header.to_str().map_err(|_| {
                        warn!("Invalid Authorization header encoding");
                        AppError::InvalidToken
                    })?;
                    if header_str.starts_with("Bearer ") {
                        header_str[7..].to_string()
                    } else {
                        warn!("Authorization header missing Bearer prefix");
                        return Err(AppError::InvalidToken.into());
                    }
                } else {
                    return Err(AppError::MissingToken.into());
                }
            };

            // Verify token and extract user ID
            let claims = match auth_service.verify_token(&token) {
                Ok(claims) => claims,
                Err(e) => {
                    warn!("Token verification failed: {:?}", e);
                    // Clear potentially compromised session
                    session.clear();
                    return Err(e.into());
                }
            };

            let user_id = match Uuid::parse_str(&claims.sub) {
                Ok(id) => id,
                Err(_) => {
                    warn!("Invalid UUID in token claims");
                    session.clear();
                    return Err(AppError::InvalidToken.into());
                }
            };

            // Cross-validate session user ID with token user ID if session exists
            if let Ok(Some(session_user_id)) = session.get::<String>("user_id") {
                if session_user_id != user_id.to_string() {
                    warn!("Session user ID mismatch with token user ID");
                    session.clear();
                    return Err(AppError::InvalidToken.into());
                }
            }

            // Security headers check
            if let Some(user_agent) = req.headers().get("User-Agent") {
                if user_agent.to_str().unwrap_or("").is_empty() {
                    warn!("Empty User-Agent header detected for user: {}", user_id);
                }
            }

            req.extensions_mut().insert(user_id);

            let res = service.call(req).await?;
            Ok(res)
        })
    }
}
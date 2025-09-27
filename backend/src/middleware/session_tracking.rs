use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use sea_orm::DatabaseConnection;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use uuid::Uuid;
use tracing::{warn, error};

use crate::utils::session_tracker::SessionTracker;

pub struct SessionTrackingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for SessionTrackingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionTrackingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionTrackingMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct SessionTrackingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionTrackingMiddlewareService<S>
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
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let request_path = req.path().to_string();
            let _request_method = req.method().as_str().to_string();

            // Security: Block suspicious requests
            if request_path.contains("..") || request_path.contains("//") {
                warn!("Suspicious path traversal attempt: {}", request_path);
                return Err(actix_web::error::ErrorBadRequest("Invalid request path").into());
            }

            // Check if user is authenticated (user_id is set by auth middleware)
            if let Some(user_id) = req.extensions().get::<Uuid>().copied() {
                // Get database connection from app data
                if let Some(db) = req.app_data::<actix_web::web::Data<DatabaseConnection>>() {
                    // Extract data from request before spawning
                    let db_clone = db.get_ref().clone();
                    let request = req.request();

                    // Extract the required data to avoid Send issues
                    let ip_address = SessionTracker::extract_ip_address_static(request);
                    let user_agent = SessionTracker::extract_user_agent_static(request);

                    // Security: Validate user agent
                    if user_agent.is_empty() || user_agent.len() > 500 {
                        warn!("Suspicious user agent for user {}: {}", user_id, user_agent);
                    }

                    // Track session in background without blocking request
                    tokio::spawn(async move {
                        if let Err(e) = SessionTracker::track_session_with_data(&db_clone, user_id, ip_address, user_agent).await {
                            error!("Failed to track session for user {}: {:?}", user_id, e);
                        }
                    });
                } else {
                    error!("Database connection not available for session tracking");
                }
            }

            // Continue with the request
            service.call(req).await
        })
    }
}
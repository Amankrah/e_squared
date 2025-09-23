use actix_web::HttpRequest;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;
use std::net::IpAddr;

use crate::models::user_session::{ActiveModel as UserSessionActiveModel, Entity as UserSessionEntity};
use crate::utils::errors::AppError;
use crate::utils::geolocation::GeolocationService;

pub struct SessionTracker;

impl SessionTracker {
    /// Track a user session by creating or updating a session record
    #[allow(dead_code)]
    pub async fn track_session(
        db: &DatabaseConnection,
        user_id: Uuid,
        req: &HttpRequest,
    ) -> Result<(), AppError> {
        let ip_address = Self::extract_ip_address(req);
        let user_agent = Self::extract_user_agent(req);
        Self::track_session_with_data(db, user_id, ip_address, user_agent).await
    }

    /// Track a user session with extracted data (for middleware)
    pub async fn track_session_with_data(
        db: &DatabaseConnection,
        user_id: Uuid,
        ip_address: String,
        user_agent: String,
    ) -> Result<(), AppError> {
        let device_info = Self::extract_device_info(&user_agent);
        let location = Self::resolve_location(&ip_address).await;

        // Check if a session already exists for this IP and user agent combination
        let existing_session = UserSessionEntity::find()
            .filter(crate::models::user_session::Column::UserId.eq(user_id))
            .filter(crate::models::user_session::Column::IpAddress.eq(&ip_address))
            .filter(crate::models::user_session::Column::UserAgent.eq(&user_agent))
            .one(db)
            .await
            .map_err(AppError::DatabaseError)?;

        if let Some(session) = existing_session {
            // Update existing session's last activity
            let mut session_model: UserSessionActiveModel = session.into();
            session_model.last_activity = Set(Utc::now());
            session_model.expires_at = Set(Utc::now() + Duration::days(30));

            session_model.update(db).await
                .map_err(AppError::DatabaseError)?;
        } else {
            // Create new session record
            let session_token = Self::generate_session_token();

            let new_session = UserSessionActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                session_token: Set(session_token),
                device_info: Set(device_info),
                ip_address: Set(ip_address),
                location: Set(location),
                user_agent: Set(user_agent),
                is_current: Set(false), // Will be set by session management handlers
                last_activity: Set(Utc::now()),
                created_at: Set(Utc::now()),
                expires_at: Set(Utc::now() + Duration::days(30)),
            };

            new_session.insert(db).await
                .map_err(AppError::DatabaseError)?;
        }

        // Clean up expired sessions for this user
        Self::cleanup_expired_sessions(db, user_id).await?;

        Ok(())
    }

    /// Extract IP address from request (static version for middleware)
    pub fn extract_ip_address_static(req: &HttpRequest) -> String {
        Self::extract_ip_address(req)
    }

    /// Extract user agent from request (static version for middleware)
    pub fn extract_user_agent_static(req: &HttpRequest) -> String {
        Self::extract_user_agent(req)
    }

    /// Extract IP address from request
    fn extract_ip_address(req: &HttpRequest) -> String {
        // Check for forwarded headers first (for proxies/load balancers)
        if let Some(forwarded_for) = req.headers().get("X-Forwarded-For") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    if let Ok(_) = first_ip.trim().parse::<IpAddr>() {
                        return first_ip.trim().to_string();
                    }
                }
            }
        }

        if let Some(real_ip) = req.headers().get("X-Real-IP") {
            if let Ok(ip_str) = real_ip.to_str() {
                if let Ok(_) = ip_str.parse::<IpAddr>() {
                    return ip_str.to_string();
                }
            }
        }

        // Fall back to connection info
        if let Some(peer_addr) = req.peer_addr() {
            peer_addr.ip().to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Extract user agent from request
    fn extract_user_agent(req: &HttpRequest) -> String {
        req.headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Extract device info from user agent
    fn extract_device_info(user_agent: &str) -> String {
        let ua = user_agent.to_lowercase();

        if ua.contains("mobile") || ua.contains("android") || ua.contains("iphone") {
            "Mobile Device".to_string()
        } else if ua.contains("tablet") || ua.contains("ipad") {
            "Tablet".to_string()
        } else {
            "Desktop/Laptop".to_string()
        }
    }

    /// Resolve location from IP address using geolocation service
    async fn resolve_location(ip_address: &str) -> Option<String> {
        let geo_service = GeolocationService::new();
        geo_service.resolve_location(ip_address).await
    }

    /// Generate a unique session token
    fn generate_session_token() -> String {
        use rand::{distributions::Alphanumeric, Rng};
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }

    /// Clean up expired sessions for a user
    async fn cleanup_expired_sessions(
        db: &DatabaseConnection,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        UserSessionEntity::delete_many()
            .filter(crate::models::user_session::Column::UserId.eq(user_id))
            .filter(crate::models::user_session::Column::ExpiresAt.lt(Utc::now()))
            .exec(db)
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(())
    }
}
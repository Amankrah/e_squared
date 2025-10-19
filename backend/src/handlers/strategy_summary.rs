use actix_web::{web, HttpRequest, HttpResponse, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, PaginatorTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;

use crate::handlers::AuthService;
use crate::utils::errors::AppError;
use crate::models::dca_strategy::Entity as DCAStrategyEntity;
use crate::models::sma_crossover_strategy::Entity as SMACrossoverStrategyEntity;
use crate::models::grid_trading_strategy::Entity as GridTradingStrategyEntity;

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyTypeSummary {
    pub strategy_type: String,
    pub count: u32,
    pub has_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStrategySummary {
    pub authenticated: bool,
    pub strategy_types: Vec<StrategyTypeSummary>,
    pub total_strategies: u32,
    pub total_active: u32,
}

/// Get user's strategy type summary with optional authentication
/// This endpoint returns which types of strategies a user has without loading full strategy data
pub async fn get_user_strategy_summary(
    db: web::Data<Arc<DatabaseConnection>>,
    auth_service: web::Data<AuthService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Try to get the token from cookies
    let token = req
        .cookie("auth_token")
        .and_then(|cookie| Some(cookie.value().to_string()));

    match token {
        Some(token_value) => {
            // Try to verify the token
            match auth_service.verify_token(&token_value) {
                Ok(claims) => {
                    // Token is valid, try to get the user_id
                    match uuid::Uuid::parse_str(&claims.sub) {
                        Ok(user_id) => {
                            // User is authenticated, get their strategy summary
                            let summary = get_authenticated_user_summary(db, user_id).await?;
                            Ok(HttpResponse::Ok().json(summary))
                        }
                        Err(_) => {
                            // Invalid UUID in token - return unauthenticated response
                            Ok(HttpResponse::Ok().json(get_unauthenticated_summary()))
                        }
                    }
                }
                Err(_) => {
                    // Token is invalid or expired - return unauthenticated response
                    Ok(HttpResponse::Ok().json(get_unauthenticated_summary()))
                }
            }
        }
        None => {
            // No token found - return unauthenticated response
            Ok(HttpResponse::Ok().json(get_unauthenticated_summary()))
        }
    }
}

async fn get_authenticated_user_summary(
    db: web::Data<Arc<DatabaseConnection>>,
    user_id: Uuid,
) -> Result<UserStrategySummary, AppError> {
    let mut strategy_types = Vec::new();
    let mut total_strategies = 0u32;
    let mut total_active = 0u32;

    // Get DCA strategy counts
    let dca_count = DCAStrategyEntity::find()
        .filter(crate::models::dca_strategy::Column::UserId.eq(user_id))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    let dca_active_count = DCAStrategyEntity::find()
        .filter(crate::models::dca_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::dca_strategy::Column::Status.eq("active"))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    if dca_count > 0 {
        strategy_types.push(StrategyTypeSummary {
            strategy_type: "dca".to_string(),
            count: dca_count,
            has_active: dca_active_count > 0,
        });
        total_strategies += dca_count;
        total_active += dca_active_count;
    }

    // Get SMA Crossover strategy counts
    let sma_count = SMACrossoverStrategyEntity::find()
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    let sma_active_count = SMACrossoverStrategyEntity::find()
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::sma_crossover_strategy::Column::Status.eq("active"))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    if sma_count > 0 {
        strategy_types.push(StrategyTypeSummary {
            strategy_type: "sma_crossover".to_string(),
            count: sma_count,
            has_active: sma_active_count > 0,
        });
        total_strategies += sma_count;
        total_active += sma_active_count;
    }

    // Get Grid Trading strategy counts
    let grid_count = GridTradingStrategyEntity::find()
        .filter(crate::models::grid_trading_strategy::Column::UserId.eq(user_id))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    let grid_active_count = GridTradingStrategyEntity::find()
        .filter(crate::models::grid_trading_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::grid_trading_strategy::Column::Status.eq("active"))
        .count(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)? as u32;

    if grid_count > 0 {
        strategy_types.push(StrategyTypeSummary {
            strategy_type: "grid_trading".to_string(),
            count: grid_count,
            has_active: grid_active_count > 0,
        });
        total_strategies += grid_count;
        total_active += grid_active_count;
    }

    Ok(UserStrategySummary {
        authenticated: true,
        strategy_types,
        total_strategies,
        total_active,
    })
}

fn get_unauthenticated_summary() -> UserStrategySummary {
    UserStrategySummary {
        authenticated: false,
        strategy_types: vec![],
        total_strategies: 0,
        total_active: 0,
    }
}
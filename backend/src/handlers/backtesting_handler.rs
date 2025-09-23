use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::{info, error};

use crate::backtesting::{BacktestEngine, BacktestConfig, BacktestResult};
use crate::strategies::{StrategyRegistry, StrategyInfo};
use crate::exchange_connectors::types::KlineInterval;
use crate::utils::errors::AppError;
use crate::handlers::auth::Claims;

#[derive(Debug, Deserialize)]
pub struct RunBacktestRequest {
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: Decimal,
    pub strategy_name: String,
    pub strategy_parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct BacktestResponse {
    pub backtest_id: String,
    pub config: BacktestConfig,
    pub result: BacktestResult,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct StrategiesListResponse {
    pub strategies: Vec<StrategyInfo>,
}

/// Get list of available strategies
pub async fn get_strategies(
    _req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let strategy_names = StrategyRegistry::list_strategies();
    let mut strategies = Vec::new();

    for name in strategy_names {
        match StrategyRegistry::get_strategy_info(name) {
            Ok(info) => strategies.push(info),
            Err(e) => {
                error!("Failed to get strategy info for {}: {}", name, e);
            }
        }
    }

    Ok(HttpResponse::Ok().json(StrategiesListResponse { strategies }))
}

/// Get detailed information about a specific strategy
pub async fn get_strategy_info(
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let strategy_name = path.into_inner();
    let strategy_info = StrategyRegistry::get_strategy_info(&strategy_name)?;
    Ok(HttpResponse::Ok().json(strategy_info))
}

/// Run a backtest
pub async fn run_backtest(
    req: HttpRequest,
    body: web::Json<RunBacktestRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user from JWT claims
    let claims = req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    info!("Running backtest for user {} with strategy {}", user_id, body.strategy_name);

    // Validate inputs
    if body.start_time >= body.end_time {
        return Err(AppError::BadRequest("Start time must be before end time".to_string()));
    }

    if body.initial_balance <= Decimal::ZERO {
        return Err(AppError::BadRequest("Initial balance must be positive".to_string()));
    }

    // Create strategy instance
    let strategy = StrategyRegistry::create_strategy(&body.strategy_name)?;

    // Validate strategy parameters
    strategy.validate_parameters(&body.strategy_parameters)?;

    // Create backtest config
    let config = BacktestConfig {
        symbol: body.symbol.clone(),
        interval: body.interval.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
        initial_balance: body.initial_balance,
        strategy_name: body.strategy_name.clone(),
        strategy_parameters: body.strategy_parameters.clone(),
    };

    // Run backtest
    let engine = BacktestEngine::new();
    let result = engine.run_backtest(config.clone(), strategy).await?;

    let response = BacktestResponse {
        backtest_id: Uuid::new_v4().to_string(),
        config,
        result,
        user_id: user_id.to_string(),
        created_at: Utc::now(),
    };

    info!("Backtest completed successfully for user {}", user_id);

    Ok(HttpResponse::Ok().json(response))
}

/// Validate strategy parameters without running a backtest
pub async fn validate_strategy_parameters(
    body: web::Json<serde_json::Value>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let strategy_name = path.into_inner();
    let strategy = StrategyRegistry::create_strategy(&strategy_name)?;

    strategy.validate_parameters(&body)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "valid": true,
        "message": "Parameters are valid"
    })))
}

/// Configure routes for backtesting
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/backtesting")
            .route("/strategies", web::get().to(get_strategies))
            .route("/strategies/{strategy_name}", web::get().to(get_strategy_info))
            .route("/strategies/{strategy_name}/validate", web::post().to(validate_strategy_parameters))
            .route("/run", web::post().to(run_backtest))
    );
}
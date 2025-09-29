use actix_web::{web, HttpResponse, HttpRequest, HttpMessage};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

use crate::backtesting::{
    BacktestEngine, BacktestConfig, BacktestRequest, BinanceFetcher,
    get_cache
};
use crate::exchange_connectors::KlineInterval;
use crate::strategies::{list_all_strategies, get_strategy_metadata};
use crate::utils::errors::AppError;
use crate::handlers::backtest_management;
use crate::handlers::AuthService;
use actix_session::SessionExt;

/// Run a backtest
pub async fn run_backtest(
    db: web::Data<std::sync::Arc<sea_orm::DatabaseConnection>>,
    req: HttpRequest,
    request: web::Json<BacktestRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    // First try to get from extensions (if auth middleware set it)
    let user_id_value = if let Some(user_id) = req.extensions().get::<Uuid>().copied() {
        user_id
    } else {
        // Try to authenticate using session or token
        authenticate_user(&req).await.map_err(|e| {
            tracing::error!("Authentication failed: {:?}", e);
            AppError::Unauthorized("Authentication required. Please log in to run backtests.".to_string())
        })?
    };

    info!("User {} starting backtest for {}", user_id_value, request.symbol);
    tracing::debug!("Backtest request: {:?}", request);

    // Parse dates
    tracing::debug!("Parsing start date: {}", request.start_date);
    let start_time = DateTime::parse_from_rfc3339(&request.start_date)
        .map_err(|e| {
            tracing::error!("Failed to parse start date '{}': {}", request.start_date, e);
            AppError::BadRequest(format!("Invalid start date: {}", e))
        })?
        .with_timezone(&Utc);

    tracing::debug!("Parsing end date: {}", request.end_date);
    let end_time = DateTime::parse_from_rfc3339(&request.end_date)
        .map_err(|e| {
            tracing::error!("Failed to parse end date '{}': {}", request.end_date, e);
            AppError::BadRequest(format!("Invalid end date: {}", e))
        })?
        .with_timezone(&Utc);

    // Parse interval
    tracing::debug!("Parsing interval: {}", request.interval);
    let interval = KlineInterval::from_str(&request.interval)
        .ok_or_else(|| {
            tracing::error!("Invalid interval: {}", request.interval);
            AppError::BadRequest(format!("Invalid interval: {}", request.interval))
        })?;

    // Prepare config
    let config = BacktestConfig {
        symbol: request.symbol.clone(),
        interval,
        start_time,
        end_time,
        initial_balance: request.initial_balance,
        strategy_name: request.strategy_name.clone(),
        strategy_parameters: request.strategy_parameters.clone().unwrap_or(json!({})),
        stop_loss_percentage: request.stop_loss_percentage,
        take_profit_percentage: request.take_profit_percentage,
    };

    // Create backtest name
    let backtest_name = format!(
        "{} {} {} Backtest",
        request.strategy_name,
        request.symbol,
        request.interval
    );

    // Prepare create request for database storage
    let create_request = backtest_management::CreateBacktestResultRequest {
        name: backtest_name,
        description: Some(format!(
            "Backtest of {} strategy on {} from {} to {}",
            request.strategy_name,
            request.symbol,
            request.start_date,
            request.end_date
        )),
        strategy_name: request.strategy_name.clone(),
        symbol: request.symbol.clone(),
        interval: request.interval.clone(),
        start_date: start_time,
        end_date: end_time,
        initial_balance: request.initial_balance,
        final_balance: rust_decimal::Decimal::from(0), // Will be updated after backtest
        total_return: rust_decimal::Decimal::from(0),
        total_return_percentage: rust_decimal::Decimal::from(0),
        max_drawdown: rust_decimal::Decimal::from(0),
        max_drawdown_percentage: rust_decimal::Decimal::from(0),
        sharpe_ratio: None,
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate: rust_decimal::Decimal::from(0),
        profit_factor: None,
        largest_win: rust_decimal::Decimal::from(0),
        largest_loss: rust_decimal::Decimal::from(0),
        average_win: rust_decimal::Decimal::from(0),
        average_loss: rust_decimal::Decimal::from(0),
        strategy_parameters: config.strategy_parameters.clone(),
        trades_data: json!([]),
        equity_curve: json!([]),
        drawdown_curve: json!([]),
        status: "running".to_string(),
        error_message: None,
        execution_time_ms: None,
    };

    // Save initial backtest record
    tracing::debug!("Saving initial backtest record...");
    let saved_backtest = backtest_management::save_backtest_result(
        web::Data::new(db.get_ref().clone()),
        user_id_value,
        create_request,
    ).await.map_err(|e| {
        tracing::error!("Failed to save initial backtest record: {:?}", e);
        e
    })?;
    tracing::debug!("Saved backtest with ID: {}", saved_backtest.id);

    // Run backtest
    tracing::debug!("Initializing backtest engine...");
    let engine = BacktestEngine::new();
    let start_time = std::time::Instant::now();

    tracing::debug!("Starting backtest execution...");
    match engine.run_backtest(config).await {
        Ok(result) => {
            let execution_time = start_time.elapsed().as_millis() as i64;

            // Update backtest with complete results
            let update_result = backtest_management::update_backtest_results(
                web::Data::new(db.get_ref().clone()),
                saved_backtest.id,
                &result,
                execution_time,
            ).await;

            if let Err(e) = update_result {
                tracing::warn!("Failed to update backtest results: {:?}", e);
            }

            // Log summary
            info!(
                "Backtest completed - Return: {:.2}%, Trades: {}, Execution: {}ms",
                result.metrics.total_return_percentage,
                result.metrics.total_trades,
                execution_time
            );

            // Return result with backtest ID
            let mut response = serde_json::to_value(result)?;
            if let Some(obj) = response.as_object_mut() {
                obj.insert("backtest_id".to_string(), serde_json::Value::String(saved_backtest.id.to_string()));
            }

            Ok(HttpResponse::Ok().json(response))
        },
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as i64;

            // Update backtest with error
            let update_result = backtest_management::update_backtest_status(
                web::Data::new(db.get_ref().clone()),
                saved_backtest.id,
                "failed".to_string(),
                Some(e.to_string()),
                Some(execution_time),
            ).await;

            if let Err(update_err) = update_result {
                tracing::warn!("Failed to update backtest error status: {:?}", update_err);
            }

            Err(e)
        }
    }
}

/// Fetch historical data without running a backtest
pub async fn fetch_historical_data(
    req: HttpRequest,
    query: web::Query<HistoricalDataQuery>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    // Parse dates
    let start_time = DateTime::parse_from_rfc3339(&query.start_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid start date: {}", e)))?
        .with_timezone(&Utc);

    let end_time = DateTime::parse_from_rfc3339(&query.end_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid end date: {}", e)))?
        .with_timezone(&Utc);

    // Parse interval
    let interval = KlineInterval::from_str(&query.interval)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid interval: {}", query.interval)))?;

    // Fetch data
    let fetcher = BinanceFetcher::new();
    let klines = fetcher
        .fetch_klines(&query.symbol, &interval, start_time, end_time)
        .await?;

    Ok(HttpResponse::Ok().json(json!({
        "symbol": query.symbol,
        "interval": query.interval,
        "start_date": query.start_date,
        "end_date": query.end_date,
        "count": klines.len(),
        "data": klines
    })))
}

/// Get available strategies
pub async fn list_strategies(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let strategy_details = list_all_strategies()?;

    Ok(HttpResponse::Ok().json(json!({
        "strategies": strategy_details
    })))
}

/// Get strategy details
pub async fn get_strategy_details(
    req: HttpRequest,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let strategy_name = path.into_inner();
    let info = get_strategy_metadata(&strategy_name)?;

    Ok(HttpResponse::Ok().json(info))
}

/// Get cache statistics
pub async fn get_cache_stats(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let cache = get_cache();
    let stats = cache.stats().await;

    Ok(HttpResponse::Ok().json(stats))
}

/// Clear the cache (admin only)
pub async fn clear_cache(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    // TODO: Add proper admin role checking

    let cache = get_cache();
    cache.clear().await;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Cache cleared successfully"
    })))
}

/// Get available symbols
pub async fn get_symbols(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let fetcher = BinanceFetcher::new();
    let symbols = fetcher.get_symbols().await?;

    Ok(HttpResponse::Ok().json(json!({
        "symbols": symbols
    })))
}

/// Get available intervals
pub async fn get_intervals(
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let intervals = vec![
        "1m", "3m", "5m", "15m", "30m",
        "1h", "2h", "4h", "6h", "8h", "12h",
        "1d", "3d", "1w", "1M"
    ];

    Ok(HttpResponse::Ok().json(json!({
        "intervals": intervals
    })))
}

/// Validate backtest parameters
pub async fn validate_backtest(
    req: HttpRequest,
    request: web::Json<BacktestRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions for authentication
    let _user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    // Validate symbol
    BinanceFetcher::validate_symbol(&request.symbol)?;

    // Parse and validate dates
    let start_time = DateTime::parse_from_rfc3339(&request.start_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid start date: {}", e)))?
        .with_timezone(&Utc);

    let end_time = DateTime::parse_from_rfc3339(&request.end_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid end date: {}", e)))?
        .with_timezone(&Utc);

    if start_time >= end_time {
        return Err(AppError::BadRequest("Start date must be before end date".to_string()));
    }

    // Validate interval
    let _interval = KlineInterval::from_str(&request.interval)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid interval: {}", request.interval)))?;

    // Validate strategy
    let _strategy_info = get_strategy_metadata(&request.strategy_name)?;

    // Validate balance
    if request.initial_balance <= Decimal::ZERO {
        return Err(AppError::BadRequest("Initial balance must be positive".to_string()));
    }

    Ok(HttpResponse::Ok().json(json!({
        "valid": true,
        "message": "All parameters are valid"
    })))
}

/// Query parameters for historical data
#[derive(Debug, Deserialize)]
pub struct HistoricalDataQuery {
    pub symbol: String,
    pub interval: String,
    pub start_date: String,
    pub end_date: String,
}

// Backtest result handlers are now in backtest_management module

/// Authenticate user from session or Authorization header
async fn authenticate_user(req: &HttpRequest) -> Result<Uuid, AppError> {
    // Try token-based authentication first (from Authorization header)
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];

                // Get auth service from app data
                if let Some(auth_service) = req.app_data::<web::Data<AuthService>>() {
                    // Verify token
                    if let Ok(claims) = auth_service.verify_token(token) {
                        if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                            return Ok(user_id);
                        }
                    }
                }
            }
        }
    }

    // Try cookie-based authentication
    if let Ok(cookies) = req.cookies() {
        if let Some(cookie) = cookies.iter().find(|c| c.name() == "auth_token") {
            let token = cookie.value();

            // Get auth service from app data
            if let Some(auth_service) = req.app_data::<web::Data<AuthService>>() {
                // Verify token
                if let Ok(claims) = auth_service.verify_token(token) {
                    if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                        return Ok(user_id);
                    }
                }
            }
        }
    }

    // Try session-based authentication last (to avoid borrow conflicts)
    let session = req.get_session();
    if let Ok(Some(user_id_str)) = session.get::<String>("user_id") {
        if let Ok(Some(authenticated)) = session.get::<bool>("authenticated") {
            if authenticated {
                if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                    return Ok(user_id);
                }
            }
        }
    }

    Err(AppError::MissingToken)
}

/// Configure backtesting routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/backtesting")
            .route("/run", web::post().to(run_backtest))
            .route("/validate", web::post().to(validate_backtest))
            .route("/results", web::get().to(backtest_management::get_user_backtest_results))
            .route("/results/{backtest_id}", web::get().to(backtest_management::get_backtest_result_detail))
            .route("/results/{backtest_id}", web::delete().to(backtest_management::delete_backtest_result))
            .route("/historical", web::get().to(fetch_historical_data))
            .route("/strategies", web::get().to(list_strategies))
            .route("/strategies/{name}", web::get().to(get_strategy_details))
            .route("/symbols", web::get().to(get_symbols))
            .route("/intervals", web::get().to(get_intervals))
            .route("/cache/stats", web::get().to(get_cache_stats))
            .route("/cache/clear", web::post().to(clear_cache))
    );
}
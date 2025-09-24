use actix_web::{web, HttpResponse};
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
use crate::strategies::StrategyRegistry;
use crate::utils::errors::AppError;

/// Run a backtest
pub async fn run_backtest(
    user_id: web::ReqData<Uuid>,
    request: web::Json<BacktestRequest>,
) -> Result<HttpResponse, AppError> {
    info!("User {} starting backtest for {}", user_id.into_inner(), request.symbol);

    // Parse dates
    let start_time = DateTime::parse_from_rfc3339(&request.start_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid start date: {}", e)))?
        .with_timezone(&Utc);

    let end_time = DateTime::parse_from_rfc3339(&request.end_date)
        .map_err(|e| AppError::BadRequest(format!("Invalid end date: {}", e)))?
        .with_timezone(&Utc);

    // Parse interval
    let interval = KlineInterval::from_str(&request.interval)
        .ok_or_else(|| AppError::BadRequest(format!("Invalid interval: {}", request.interval)))?;

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

    // Run backtest
    let engine = BacktestEngine::new();
    let result = engine.run_backtest(config).await?;

    // Log summary
    info!(
        "Backtest completed - Return: {:.2}%, Trades: {}, Execution: {}ms",
        result.metrics.total_return_percentage,
        result.metrics.total_trades,
        result.execution_time_ms
    );

    Ok(HttpResponse::Ok().json(result))
}

/// Fetch historical data without running a backtest
pub async fn fetch_historical_data(
    _user_id: web::ReqData<Uuid>,
    query: web::Query<HistoricalDataQuery>,
) -> Result<HttpResponse, AppError> {
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
    _user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let strategies = StrategyRegistry::list_strategies();

    let mut strategy_details = Vec::new();
    for name in strategies {
        if let Ok(info) = StrategyRegistry::get_strategy_info(name) {
            strategy_details.push(info);
        }
    }

    Ok(HttpResponse::Ok().json(json!({
        "strategies": strategy_details
    })))
}

/// Get strategy details
pub async fn get_strategy_details(
    _user_id: web::ReqData<Uuid>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let strategy_name = path.into_inner();
    let info = StrategyRegistry::get_strategy_info(&strategy_name)?;

    Ok(HttpResponse::Ok().json(info))
}

/// Get cache statistics
pub async fn get_cache_stats(
    _user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let cache = get_cache();
    let stats = cache.stats().await;

    Ok(HttpResponse::Ok().json(stats))
}

/// Clear the cache (admin only)
pub async fn clear_cache(
    _user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    // TODO: Add proper admin role checking

    let cache = get_cache();
    cache.clear().await;

    Ok(HttpResponse::Ok().json(json!({
        "message": "Cache cleared successfully"
    })))
}

/// Get available symbols
pub async fn get_symbols(
    _user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
    let fetcher = BinanceFetcher::new();
    let symbols = fetcher.get_symbols().await?;

    Ok(HttpResponse::Ok().json(json!({
        "symbols": symbols
    })))
}

/// Get available intervals
pub async fn get_intervals(
    _user_id: web::ReqData<Uuid>,
) -> Result<HttpResponse, AppError> {
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
    _user_id: web::ReqData<Uuid>,
    request: web::Json<BacktestRequest>,
) -> Result<HttpResponse, AppError> {
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
    let _strategy_info = StrategyRegistry::get_strategy_info(&request.strategy_name)?;

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

/// Configure backtesting routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/backtesting")
            .route("/run", web::post().to(run_backtest))
            .route("/validate", web::post().to(validate_backtest))
            .route("/historical", web::get().to(fetch_historical_data))
            .route("/strategies", web::get().to(list_strategies))
            .route("/strategies/{name}", web::get().to(get_strategy_details))
            .route("/symbols", web::get().to(get_symbols))
            .route("/intervals", web::get().to(get_intervals))
            .route("/cache/stats", web::get().to(get_cache_stats))
            .route("/cache/clear", web::post().to(clear_cache))
    );
}
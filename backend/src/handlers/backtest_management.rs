use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set, PaginatorTrait
};
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use actix_session::SessionExt;
use crate::handlers::AuthService;

use crate::models::backtest_result::{
    Entity as BacktestResultEntity,
    ActiveModel as BacktestResultActiveModel,
    BacktestResultResponse,
    BacktestResultDetailResponse,
    Model as BacktestResultModel,
};
use crate::utils::errors::AppError;

/// Authenticate user from various sources (token, cookie, session)
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

/// Get user's backtest results with pagination
pub async fn get_user_backtest_results(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    query: web::Query<BacktestListQuery>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware) or authenticate
    let user_id_value = if let Some(user_id) = req.extensions().get::<Uuid>().copied() {
        user_id
    } else {
        // Try to authenticate using session or token
        authenticate_user(&req).await.map_err(|e| {
            tracing::error!("Authentication failed for backtest results: {:?}", e);
            AppError::Unauthorized("Authentication required".to_string())
        })?
    };

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1); // Max 100, min 1
    let offset = (page - 1) * limit;

    // Build query with optional filters
    let mut select = BacktestResultEntity::find()
        .filter(crate::models::backtest_result::Column::UserId.eq(user_id_value));

    if let Some(ref strategy) = query.strategy_name {
        select = select.filter(crate::models::backtest_result::Column::StrategyName.eq(strategy));
    }

    if let Some(ref symbol) = query.symbol {
        select = select.filter(crate::models::backtest_result::Column::Symbol.eq(symbol));
    }

    if let Some(ref status) = query.status {
        select = select.filter(crate::models::backtest_result::Column::Status.eq(status));
    }

    // Get total count for pagination
    let total = select.clone().count(db.get_ref().as_ref()).await
        .map_err(AppError::DatabaseError)?;

    // Get paginated results
    let results = select
        .order_by_desc(crate::models::backtest_result::Column::CreatedAt)
        .offset(Some(offset as u64))
        .limit(Some(limit as u64))
        .all(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let backtest_responses: Vec<BacktestResultResponse> = results.into_iter()
        .map(BacktestResultResponse::from)
        .collect();

    let response = BacktestListResponse {
        results: backtest_responses,
        pagination: PaginationInfo {
            page,
            limit,
            total: total as u32,
            total_pages: ((total as f64) / (limit as f64)).ceil() as u32,
        },
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get specific backtest result with full details
pub async fn get_backtest_result_detail(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id_value = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| {
            tracing::error!("User ID not found in request extensions - authentication required");
            AppError::Unauthorized("Authentication required".to_string())
        })?;
    let backtest_id = path.into_inner();

    let result = BacktestResultEntity::find()
        .filter(crate::models::backtest_result::Column::Id.eq(backtest_id))
        .filter(crate::models::backtest_result::Column::UserId.eq(user_id_value))
        .one(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Backtest result not found".to_string()))?;

    let response = BacktestResultDetailResponse::from(result);
    Ok(HttpResponse::Ok().json(response))
}

/// Delete backtest result
pub async fn delete_backtest_result(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware) or authenticate
    let user_id_value = if let Some(user_id) = req.extensions().get::<Uuid>().copied() {
        user_id
    } else {
        // Try to authenticate using session or token
        authenticate_user(&req).await.map_err(|e| {
            tracing::error!("Authentication failed for delete backtest: {:?}", e);
            AppError::Unauthorized("Authentication required".to_string())
        })?
    };
    let backtest_id = path.into_inner();

    // First verify the backtest belongs to the user
    let result = BacktestResultEntity::find()
        .filter(crate::models::backtest_result::Column::Id.eq(backtest_id))
        .filter(crate::models::backtest_result::Column::UserId.eq(user_id_value))
        .one(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Backtest result not found".to_string()))?;

    // Delete the backtest result
    let active_model: BacktestResultActiveModel = result.into();
    active_model.delete(db.get_ref().as_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Backtest result deleted successfully"
    })))
}

/// Create/save a new backtest result
pub async fn save_backtest_result(
    db: web::Data<Arc<DatabaseConnection>>,
    user_id: Uuid,
    request: CreateBacktestResultRequest,
) -> Result<BacktestResultModel, AppError> {
    let backtest_id = Uuid::new_v4();

    let new_backtest = BacktestResultActiveModel {
        id: Set(backtest_id),
        user_id: Set(user_id),
        name: Set(request.name),
        description: Set(request.description),
        strategy_name: Set(request.strategy_name),
        symbol: Set(request.symbol),
        interval: Set(request.interval),
        start_date: Set(request.start_date),
        end_date: Set(request.end_date),
        initial_balance: Set(request.initial_balance),
        final_balance: Set(request.final_balance),
        total_return: Set(request.total_return),
        total_return_percentage: Set(request.total_return_percentage),
        max_drawdown: Set(request.max_drawdown),
        max_drawdown_percentage: Set(request.max_drawdown_percentage),
        sharpe_ratio: Set(request.sharpe_ratio),
        total_trades: Set(request.total_trades),
        winning_trades: Set(request.winning_trades),
        losing_trades: Set(request.losing_trades),
        win_rate: Set(request.win_rate),
        profit_factor: Set(request.profit_factor),
        largest_win: Set(request.largest_win),
        largest_loss: Set(request.largest_loss),
        average_win: Set(request.average_win),
        average_loss: Set(request.average_loss),
        strategy_parameters: Set(request.strategy_parameters),
        trades_data: Set(request.trades_data),
        equity_curve: Set(request.equity_curve),
        drawdown_curve: Set(request.drawdown_curve),
        status: Set(request.status),
        error_message: Set(request.error_message),
        execution_time_ms: Set(request.execution_time_ms),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    // Insert the backtest result
    BacktestResultEntity::insert(new_backtest)
        .exec_without_returning(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Fetch and return the created backtest
    let result = BacktestResultEntity::find_by_id(backtest_id)
        .one(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::InternalServerError)?;

    Ok(result)
}

/// Update backtest result status (for running backtests)
pub async fn update_backtest_status(
    db: web::Data<Arc<DatabaseConnection>>,
    backtest_id: Uuid,
    status: String,
    error_message: Option<String>,
    execution_time_ms: Option<i64>,
) -> Result<(), AppError> {
    let result = BacktestResultEntity::find_by_id(backtest_id)
        .one(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Backtest result not found".to_string()))?;

    let mut active_model: BacktestResultActiveModel = result.into();
    active_model.status = Set(status);
    active_model.updated_at = Set(Utc::now());

    if let Some(error) = error_message {
        active_model.error_message = Set(Some(error));
    }

    if let Some(exec_time) = execution_time_ms {
        active_model.execution_time_ms = Set(Some(exec_time));
    }

    active_model.update(db.get_ref().as_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(())
}

/// Update backtest result with complete results from engine
pub async fn update_backtest_results(
    db: web::Data<Arc<DatabaseConnection>>,
    backtest_id: Uuid,
    engine_result: &crate::backtesting::types::BacktestResult,
    execution_time_ms: i64,
) -> Result<(), AppError> {
    let result = BacktestResultEntity::find_by_id(backtest_id)
        .one(db.get_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Backtest result not found".to_string()))?;

    // Store initial_balance before moving result
    let initial_balance = result.initial_balance;
    let mut active_model: BacktestResultActiveModel = result.into();

    // Update with engine results
    active_model.final_balance = Set(engine_result.metrics.final_portfolio_value);
    active_model.total_return = Set(engine_result.metrics.total_return);
    active_model.total_return_percentage = Set(engine_result.metrics.total_return_percentage);
    active_model.max_drawdown = Set(engine_result.metrics.max_drawdown);
    active_model.max_drawdown_percentage = Set(engine_result.metrics.max_drawdown / initial_balance * rust_decimal::Decimal::from(100));
    active_model.sharpe_ratio = Set(engine_result.metrics.sharpe_ratio);
    active_model.total_trades = Set(engine_result.metrics.total_trades as i32);
    active_model.winning_trades = Set(engine_result.metrics.winning_trades as i32);
    active_model.losing_trades = Set(engine_result.metrics.losing_trades as i32);
    active_model.win_rate = Set(engine_result.metrics.win_rate);
    active_model.profit_factor = Set(engine_result.metrics.profit_factor);
    active_model.average_win = Set(engine_result.metrics.average_win);
    active_model.average_loss = Set(engine_result.metrics.average_loss);

    // Calculate largest win/loss from trades
    let mut largest_win = rust_decimal::Decimal::from(0);
    let mut largest_loss = rust_decimal::Decimal::from(0);

    for trade in &engine_result.trades {
        if let Some(pnl) = trade.pnl {
            if pnl > largest_win {
                largest_win = pnl;
            }
            if pnl < largest_loss {
                largest_loss = pnl;
            }
        }
    }

    active_model.largest_win = Set(largest_win);
    active_model.largest_loss = Set(largest_loss);

    // Store detailed data as JSON
    active_model.trades_data = Set(serde_json::to_value(&engine_result.trades).unwrap_or(serde_json::json!([])));
    active_model.equity_curve = Set(serde_json::to_value(&engine_result.performance_chart).unwrap_or(serde_json::json!([])));
    active_model.drawdown_curve = Set(serde_json::json!([])); // Could be calculated from performance_chart if needed

    // Update status and timing
    active_model.status = Set("completed".to_string());
    active_model.execution_time_ms = Set(Some(execution_time_ms));
    active_model.updated_at = Set(Utc::now());

    active_model.update(db.get_ref().as_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(())
}

// Request/Response DTOs
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc as ChronoUtc};

#[derive(Debug, Deserialize)]
pub struct BacktestListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub strategy_name: Option<String>,
    pub symbol: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BacktestListResponse {
    pub results: Vec<BacktestResultResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateBacktestResultRequest {
    pub name: String,
    pub description: Option<String>,
    pub strategy_name: String,
    pub symbol: String,
    pub interval: String,
    pub start_date: DateTime<ChronoUtc>,
    pub end_date: DateTime<ChronoUtc>,
    pub initial_balance: Decimal,
    pub final_balance: Decimal,
    pub total_return: Decimal,
    pub total_return_percentage: Decimal,
    pub max_drawdown: Decimal,
    pub max_drawdown_percentage: Decimal,
    pub sharpe_ratio: Option<Decimal>,
    pub total_trades: i32,
    pub winning_trades: i32,
    pub losing_trades: i32,
    pub win_rate: Decimal,
    pub profit_factor: Option<Decimal>,
    pub largest_win: Decimal,
    pub largest_loss: Decimal,
    pub average_win: Decimal,
    pub average_loss: Decimal,
    pub strategy_parameters: serde_json::Value,
    pub trades_data: serde_json::Value,
    pub equity_curve: serde_json::Value,
    pub drawdown_curve: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i64>,
}
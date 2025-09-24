use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;

use crate::models::macd_strategy::{
    Entity as MACDStrategyEntity,
    ActiveModel as MACDStrategyActiveModel,
    ExecutionEntity as MACDExecutionEntity,
    CreateMACDStrategyRequest, UpdateMACDStrategyRequest,
    MACDStrategyResponse, MACDStrategiesResponse, MACDExecutionResponse,
    MACDStatus,
};
use crate::services::MarketDataService;
use crate::utils::errors::AppError;

/// Create a new MACD strategy
pub async fn create_macd_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateMACDStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Check if user already has a strategy with this name
    let existing_strategy = MACDStrategyEntity::find()
        .filter(crate::models::macd_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::macd_strategy::Column::Name.eq(&body.name))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_strategy.is_some() {
        return Err(AppError::BadRequest("Strategy with this name already exists".to_string()));
    }

    // Validate MACDStrategyConfig
    body.config.validate().map_err(|e| AppError::BadRequest(format!("Invalid MACDStrategyConfig: {}", e)))?;

    // Serialize MACDStrategyConfig to JSON
    let config_json = serde_json::to_string(&body.config)
        .map_err(|e| AppError::BadRequest(format!("Failed to serialize MACDStrategyConfig: {}", e)))?;

    // Create the strategy
    let new_strategy = MACDStrategyActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(body.name.clone()),
        asset_symbol: Set(body.asset_symbol.to_uppercase()),
        status: Set(MACDStatus::Active.into()),
        config_json: Set(config_json),
        total_invested: Set(Decimal::ZERO),
        total_purchased: Set(Decimal::ZERO),
        average_buy_price: Set(None),
        current_position: Set(0),
        total_trades: Set(0),
        winning_trades: Set(0),
        losing_trades: Set(0),
        realized_pnl: Set(Decimal::ZERO),
        unrealized_pnl: Set(None),
        current_streak: Set(0),
        max_drawdown: Set(None),
        last_macd_value: Set(None),
        last_signal_value: Set(None),
        last_histogram_value: Set(None),
        last_signal_time: Set(None),
        trend_state: Set(None),
        last_execution_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let strategy = new_strategy.insert(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = MACDStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_macd_config().map_err(|e| AppError::BadRequest(e))?,
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_position: strategy.current_position,
        total_trades: strategy.total_trades,
        winning_trades: strategy.winning_trades,
        losing_trades: strategy.losing_trades,
        win_rate: strategy.calculate_win_rate(),
        realized_pnl: strategy.realized_pnl,
        unrealized_pnl: strategy.unrealized_pnl,
        total_pnl: strategy.calculate_total_pnl(),
        current_streak: strategy.current_streak,
        max_drawdown: strategy.max_drawdown,
        last_macd_value: strategy.last_macd_value,
        last_signal_value: strategy.last_signal_value,
        last_histogram_value: strategy.last_histogram_value,
        last_signal_time: strategy.last_signal_time,
        trend_state: strategy.trend_state,
        last_execution_at: strategy.last_execution_at,
        recent_executions: Vec::new(),
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Get user's MACD strategies
pub async fn get_macd_strategies(
    db: web::Data<DatabaseConnection>,
    market_service: web::Data<MarketDataService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get user's strategies
    let strategies = MACDStrategyEntity::find()
        .filter(crate::models::macd_strategy::Column::UserId.eq(user_id))
        .order_by_desc(crate::models::macd_strategy::Column::CreatedAt)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut strategy_responses = Vec::new();
    let mut total_invested = Decimal::ZERO;
    let mut total_pnl = Decimal::ZERO;
    let mut active_strategies = 0;
    let mut total_win_rate = Decimal::ZERO;

    for strategy in strategies {
        // Get recent executions for this strategy
        let recent_executions = MACDExecutionEntity::find()
            .filter(crate::models::macd_strategy::execution::Column::StrategyId.eq(strategy.id))
            .order_by_desc(crate::models::macd_strategy::execution::Column::ExecutionTimestamp)
            .limit(10)
            .all(db.get_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        let execution_responses: Vec<MACDExecutionResponse> = recent_executions.into_iter()
            .map(|exec| MACDExecutionResponse {
                id: exec.id,
                strategy_id: exec.strategy_id,
                execution_type: exec.execution_type,
                trigger_reason: exec.trigger_reason,
                amount_usd: exec.amount_usd,
                amount_asset: exec.amount_asset,
                price_at_execution: exec.price_at_execution,
                macd_value: exec.macd_value,
                signal_value: exec.signal_value,
                histogram_value: exec.histogram_value,
                signal_strength: exec.signal_strength,
                crossover_type: exec.crossover_type,
                position_before: exec.position_before,
                position_after: exec.position_after,
                realized_pnl: exec.realized_pnl,
                order_status: exec.order_status,
                execution_timestamp: exec.execution_timestamp,
                error_message: exec.error_message,
            })
            .collect();

        // Calculate current unrealized P&L if we have positions
        let unrealized_pnl = if strategy.total_purchased > Decimal::ZERO && strategy.current_position != 0 {
            match market_service.get_current_price(&strategy.asset_symbol).await {
                Ok(current_price) => {
                    if let Some(avg_price) = strategy.average_buy_price {
                        let current_value = strategy.total_purchased * current_price;
                        let invested_value = strategy.total_purchased * avg_price;
                        Some(current_value - invested_value)
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };

        // Update totals
        total_invested += strategy.total_invested;
        if let Some(pnl) = strategy.calculate_total_pnl() {
            total_pnl += pnl;
        }
        if strategy.status == String::from(MACDStatus::Active) {
            active_strategies += 1;
        }
        total_win_rate += strategy.calculate_win_rate();

        let strategy_response = MACDStrategyResponse {
            id: strategy.id,
            user_id: strategy.user_id,
            name: strategy.name.clone(),
            asset_symbol: strategy.asset_symbol.clone(),
            status: strategy.status.clone(),
            config: strategy.get_macd_config().unwrap_or_else(|_| Default::default()),
            total_invested: strategy.total_invested,
            total_purchased: strategy.total_purchased,
            average_buy_price: strategy.average_buy_price,
            current_position: strategy.current_position,
            total_trades: strategy.total_trades,
            winning_trades: strategy.winning_trades,
            losing_trades: strategy.losing_trades,
            win_rate: strategy.calculate_win_rate(),
            realized_pnl: strategy.realized_pnl,
            unrealized_pnl,
            total_pnl: strategy.calculate_total_pnl(),
            current_streak: strategy.current_streak,
            max_drawdown: strategy.max_drawdown,
            last_macd_value: strategy.last_macd_value,
            last_signal_value: strategy.last_signal_value,
            last_histogram_value: strategy.last_histogram_value,
            last_signal_time: strategy.last_signal_time,
            trend_state: strategy.trend_state,
            last_execution_at: strategy.last_execution_at,
            recent_executions: execution_responses,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        };

        strategy_responses.push(strategy_response);
    }

    let average_win_rate = if !strategy_responses.is_empty() {
        total_win_rate / Decimal::from(strategy_responses.len())
    } else {
        Decimal::ZERO
    };

    let response = MACDStrategiesResponse {
        strategies: strategy_responses,
        total_invested,
        total_pnl,
        active_strategies,
        average_win_rate,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get a specific MACD strategy
pub async fn get_macd_strategy(
    db: web::Data<DatabaseConnection>,
    market_service: web::Data<MarketDataService>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Get the strategy
    let strategy = MACDStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Get recent executions
    let recent_executions = MACDExecutionEntity::find()
        .filter(crate::models::macd_strategy::execution::Column::StrategyId.eq(strategy.id))
        .order_by_desc(crate::models::macd_strategy::execution::Column::ExecutionTimestamp)
        .limit(50)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let execution_responses: Vec<MACDExecutionResponse> = recent_executions.into_iter()
        .map(|exec| MACDExecutionResponse {
            id: exec.id,
            strategy_id: exec.strategy_id,
            execution_type: exec.execution_type,
            trigger_reason: exec.trigger_reason,
            amount_usd: exec.amount_usd,
            amount_asset: exec.amount_asset,
            price_at_execution: exec.price_at_execution,
            macd_value: exec.macd_value,
            signal_value: exec.signal_value,
            histogram_value: exec.histogram_value,
            signal_strength: exec.signal_strength,
            crossover_type: exec.crossover_type,
            position_before: exec.position_before,
            position_after: exec.position_after,
            realized_pnl: exec.realized_pnl,
            order_status: exec.order_status,
            execution_timestamp: exec.execution_timestamp,
            error_message: exec.error_message,
        })
        .collect();

    // Calculate current unrealized P&L
    let unrealized_pnl = if strategy.total_purchased > Decimal::ZERO && strategy.current_position != 0 {
        match market_service.get_current_price(&strategy.asset_symbol).await {
            Ok(current_price) => {
                if let Some(avg_price) = strategy.average_buy_price {
                    let current_value = strategy.total_purchased * current_price;
                    let invested_value = strategy.total_purchased * avg_price;
                    Some(current_value - invested_value)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let response = MACDStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_macd_config().unwrap_or_else(|_| Default::default()),
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_position: strategy.current_position,
        total_trades: strategy.total_trades,
        winning_trades: strategy.winning_trades,
        losing_trades: strategy.losing_trades,
        win_rate: strategy.calculate_win_rate(),
        realized_pnl: strategy.realized_pnl,
        unrealized_pnl,
        total_pnl: strategy.calculate_total_pnl(),
        current_streak: strategy.current_streak,
        max_drawdown: strategy.max_drawdown,
        last_macd_value: strategy.last_macd_value,
        last_signal_value: strategy.last_signal_value,
        last_histogram_value: strategy.last_histogram_value,
        last_signal_time: strategy.last_signal_time,
        trend_state: strategy.trend_state,
        last_execution_at: strategy.last_execution_at,
        recent_executions: execution_responses,
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Update a MACD strategy
pub async fn update_macd_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateMACDStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Get the strategy
    let mut strategy: MACDStrategyActiveModel = MACDStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?
        .into();

    // Verify ownership
    if let sea_orm::ActiveValue::Set(user_id_value) = &strategy.user_id {
        if *user_id_value != user_id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
    }

    // Update fields if provided
    if let Some(name) = &body.name {
        strategy.name = Set(name.clone());
    }

    // Update status if provided
    if let Some(status) = &body.status {
        strategy.status = Set(status.clone().into());
    }

    // Update config if provided
    if let Some(config) = &body.config {
        // Validate the new config
        config.validate().map_err(|e| AppError::BadRequest(format!("Invalid MACDStrategyConfig: {}", e)))?;

        // Serialize the new config to JSON
        let config_json = serde_json::to_string(&config)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize MACDStrategyConfig: {}", e)))?;

        strategy.config_json = Set(config_json);
    }

    strategy.updated_at = Set(Utc::now());

    // Save changes
    let updated_strategy = strategy.update(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = MACDStrategyResponse {
        id: updated_strategy.id,
        user_id: updated_strategy.user_id,
        name: updated_strategy.name.clone(),
        asset_symbol: updated_strategy.asset_symbol.clone(),
        status: updated_strategy.status.clone(),
        config: updated_strategy.get_macd_config().unwrap_or_else(|_| Default::default()),
        total_invested: updated_strategy.total_invested,
        total_purchased: updated_strategy.total_purchased,
        average_buy_price: updated_strategy.average_buy_price,
        current_position: updated_strategy.current_position,
        total_trades: updated_strategy.total_trades,
        winning_trades: updated_strategy.winning_trades,
        losing_trades: updated_strategy.losing_trades,
        win_rate: updated_strategy.calculate_win_rate(),
        realized_pnl: updated_strategy.realized_pnl,
        unrealized_pnl: updated_strategy.unrealized_pnl,
        total_pnl: updated_strategy.calculate_total_pnl(),
        current_streak: updated_strategy.current_streak,
        max_drawdown: updated_strategy.max_drawdown,
        last_macd_value: updated_strategy.last_macd_value,
        last_signal_value: updated_strategy.last_signal_value,
        last_histogram_value: updated_strategy.last_histogram_value,
        last_signal_time: updated_strategy.last_signal_time,
        trend_state: updated_strategy.trend_state,
        last_execution_at: updated_strategy.last_execution_at,
        recent_executions: Vec::new(),
        created_at: updated_strategy.created_at,
        updated_at: updated_strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete a MACD strategy
pub async fn delete_macd_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Get the strategy
    let strategy = MACDStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Delete the strategy (cascading delete will handle executions)
    MACDStrategyEntity::delete_by_id(strategy_id)
        .exec(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy deleted successfully"
    })))
}

/// Get MACD strategy execution statistics
pub async fn get_macd_execution_stats(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get user's strategies to calculate stats
    let strategies = MACDStrategyEntity::find()
        .filter(crate::models::macd_strategy::Column::UserId.eq(user_id))
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut total_trades = 0;
    let mut total_winning_trades = 0;
    let mut total_invested = Decimal::ZERO;
    let mut total_pnl = Decimal::ZERO;

    for strategy in strategies {
        total_trades += strategy.total_trades;
        total_winning_trades += strategy.winning_trades;
        total_invested += strategy.total_invested;
        if let Some(pnl) = strategy.calculate_total_pnl() {
            total_pnl += pnl;
        }
    }

    let overall_win_rate = if total_trades > 0 {
        Decimal::from(total_winning_trades) / Decimal::from(total_trades) * Decimal::from(100)
    } else {
        Decimal::ZERO
    };

    let stats = serde_json::json!({
        "total_trades": total_trades,
        "winning_trades": total_winning_trades,
        "losing_trades": total_trades - total_winning_trades,
        "win_rate": overall_win_rate,
        "total_invested": total_invested,
        "total_pnl": total_pnl,
        "roi_percentage": if total_invested > Decimal::ZERO {
            (total_pnl / total_invested) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    });

    Ok(HttpResponse::Ok().json(stats))
}
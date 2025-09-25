use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;

use crate::models::sma_crossover_strategy::{
    Entity as SMACrossoverStrategyEntity,
    ActiveModel as SMACrossoverStrategyActiveModel,
    ExecutionEntity as SMACrossoverExecutionEntity,
    CreateSMACrossoverStrategyRequest, UpdateSMACrossoverStrategyRequest,
    SMACrossoverStrategyResponse, SMACrossoverStrategiesResponse, SMACrossoverExecutionResponse,
};
use crate::utils::errors::AppError;

/// Create a new SMA Crossover strategy
pub async fn create_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateSMACrossoverStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Check if user already has a strategy with this name
    let existing_strategy = SMACrossoverStrategyEntity::find()
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::sma_crossover_strategy::Column::Name.eq(&body.name))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_strategy.is_some() {
        return Err(AppError::BadRequest("Strategy with this name already exists".to_string()));
    }

    // Validate SMACrossoverConfig
    body.config.validate().map_err(|e| AppError::BadRequest(format!("Invalid SMACrossoverConfig: {}", e)))?;

    // Serialize SMACrossoverConfig to JSON
    let config_json = serde_json::to_string(&body.config)
        .map_err(|e| AppError::BadRequest(format!("Failed to serialize config: {}", e)))?;

    // Create new strategy
    let now = Utc::now();
    let strategy_id = Uuid::new_v4();

    let new_strategy = SMACrossoverStrategyActiveModel {
        id: Set(strategy_id),
        user_id: Set(user_id),
        name: Set(body.name.clone()),
        asset_symbol: Set(body.asset_symbol.clone().to_uppercase()),
        status: Set("active".to_string()),
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
        last_fast_sma: Set(None),
        last_slow_sma: Set(None),
        last_signal_type: Set(None),
        last_signal_time: Set(None),
        last_execution_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let saved_strategy = new_strategy.insert(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = SMACrossoverStrategyResponse {
        id: saved_strategy.id,
        user_id: saved_strategy.user_id,
        name: saved_strategy.name.clone(),
        asset_symbol: saved_strategy.asset_symbol.clone(),
        status: saved_strategy.status.clone(),
        config: body.config.clone(),
        total_invested: saved_strategy.total_invested,
        total_purchased: saved_strategy.total_purchased,
        average_buy_price: saved_strategy.average_buy_price,
        current_position: saved_strategy.current_position,
        total_trades: saved_strategy.total_trades,
        winning_trades: saved_strategy.winning_trades,
        losing_trades: saved_strategy.losing_trades,
        win_rate: saved_strategy.calculate_win_rate(),
        realized_pnl: saved_strategy.realized_pnl,
        unrealized_pnl: saved_strategy.unrealized_pnl,
        total_pnl: saved_strategy.calculate_total_pnl(),
        current_streak: saved_strategy.current_streak,
        max_drawdown: saved_strategy.max_drawdown,
        last_fast_sma: saved_strategy.last_fast_sma,
        last_slow_sma: saved_strategy.last_slow_sma,
        sma_spread: saved_strategy.calculate_sma_spread(),
        last_signal_type: saved_strategy.last_signal_type,
        last_signal_time: saved_strategy.last_signal_time,
        last_execution_at: saved_strategy.last_execution_at,
        recent_executions: vec![], // Empty for new strategy
        created_at: saved_strategy.created_at,
        updated_at: saved_strategy.updated_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Get all SMA Crossover strategies for a user
pub async fn get_user_sma_crossover_strategies(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get all strategies for the user
    let strategies = SMACrossoverStrategyEntity::find()
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .order_by_desc(crate::models::sma_crossover_strategy::Column::CreatedAt)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut strategy_responses = Vec::new();
    let mut total_invested = Decimal::ZERO;
    let mut total_pnl = Decimal::ZERO;
    let mut active_strategies = 0;
    let mut total_win_rate = Decimal::ZERO;

    for strategy in strategies {
        let config = strategy.get_sma_crossover_config().map_err(|e| AppError::BadRequest(e))?;

        // Get recent executions (last 10)
        let recent_executions = SMACrossoverExecutionEntity::find()
            .filter(crate::models::sma_crossover_strategy::execution::Column::StrategyId.eq(strategy.id))
            .order_by_desc(crate::models::sma_crossover_strategy::execution::Column::ExecutionTimestamp)
            .limit(10)
            .all(db.get_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        let execution_responses = recent_executions.into_iter().map(|exec| {
            SMACrossoverExecutionResponse {
                id: exec.id,
                strategy_id: exec.strategy_id,
                execution_type: exec.execution_type,
                trigger_reason: exec.trigger_reason,
                amount_usd: exec.amount_usd,
                amount_asset: exec.amount_asset,
                price_at_execution: exec.price_at_execution,
                fast_sma_value: exec.fast_sma_value,
                slow_sma_value: exec.slow_sma_value,
                sma_spread: exec.sma_spread,
                signal_strength: exec.signal_strength,
                crossover_type: exec.crossover_type,
                position_before: exec.position_before,
                position_after: exec.position_after,
                realized_pnl: exec.realized_pnl,
                order_status: exec.order_status,
                execution_timestamp: exec.execution_timestamp,
                error_message: exec.error_message,
            }
        }).collect();

        let win_rate = strategy.calculate_win_rate();
        let pnl = strategy.calculate_total_pnl().unwrap_or(Decimal::ZERO);

        total_invested += strategy.total_invested;
        total_pnl += pnl;
        total_win_rate += win_rate;

        if strategy.status == "active" {
            active_strategies += 1;
        }

        let response = SMACrossoverStrategyResponse {
            id: strategy.id,
            user_id: strategy.user_id,
            name: strategy.name.clone(),
            asset_symbol: strategy.asset_symbol.clone(),
            status: strategy.status.clone(),
            config,
            total_invested: strategy.total_invested,
            total_purchased: strategy.total_purchased,
            average_buy_price: strategy.average_buy_price,
            current_position: strategy.current_position,
            total_trades: strategy.total_trades,
            winning_trades: strategy.winning_trades,
            losing_trades: strategy.losing_trades,
            win_rate,
            realized_pnl: strategy.realized_pnl,
            unrealized_pnl: strategy.unrealized_pnl,
            total_pnl: strategy.calculate_total_pnl(),
            current_streak: strategy.current_streak,
            max_drawdown: strategy.max_drawdown,
            last_fast_sma: strategy.last_fast_sma,
            last_slow_sma: strategy.last_slow_sma,
            sma_spread: strategy.calculate_sma_spread(),
            last_signal_type: strategy.last_signal_type,
            last_signal_time: strategy.last_signal_time,
            last_execution_at: strategy.last_execution_at,
            recent_executions: execution_responses,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        };

        strategy_responses.push(response);
    }

    let average_win_rate = if !strategy_responses.is_empty() {
        total_win_rate / Decimal::from(strategy_responses.len())
    } else {
        Decimal::ZERO
    };

    let response = SMACrossoverStrategiesResponse {
        strategies: strategy_responses,
        total_invested,
        total_pnl,
        active_strategies,
        average_win_rate,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get a specific SMA Crossover strategy by ID
pub async fn get_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    let strategy = SMACrossoverStrategyEntity::find_by_id(strategy_id)
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    let config = strategy.get_sma_crossover_config().map_err(|e| AppError::BadRequest(e))?;

    // Get recent executions
    let recent_executions = SMACrossoverExecutionEntity::find()
        .filter(crate::models::sma_crossover_strategy::execution::Column::StrategyId.eq(strategy.id))
        .order_by_desc(crate::models::sma_crossover_strategy::execution::Column::ExecutionTimestamp)
        .limit(20)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let execution_responses = recent_executions.into_iter().map(|exec| {
        SMACrossoverExecutionResponse {
            id: exec.id,
            strategy_id: exec.strategy_id,
            execution_type: exec.execution_type,
            trigger_reason: exec.trigger_reason,
            amount_usd: exec.amount_usd,
            amount_asset: exec.amount_asset,
            price_at_execution: exec.price_at_execution,
            fast_sma_value: exec.fast_sma_value,
            slow_sma_value: exec.slow_sma_value,
            sma_spread: exec.sma_spread,
            signal_strength: exec.signal_strength,
            crossover_type: exec.crossover_type,
            position_before: exec.position_before,
            position_after: exec.position_after,
            realized_pnl: exec.realized_pnl,
            order_status: exec.order_status,
            execution_timestamp: exec.execution_timestamp,
            error_message: exec.error_message,
        }
    }).collect();

    let response = SMACrossoverStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config,
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
        last_fast_sma: strategy.last_fast_sma,
        last_slow_sma: strategy.last_slow_sma,
        sma_spread: strategy.calculate_sma_spread(),
        last_signal_type: strategy.last_signal_type,
        last_signal_time: strategy.last_signal_time,
        last_execution_at: strategy.last_execution_at,
        recent_executions: execution_responses,
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Update an existing SMA Crossover strategy
pub async fn update_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateSMACrossoverStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Find and validate ownership
    let strategy = SMACrossoverStrategyEntity::find_by_id(strategy_id)
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Prepare update
    let mut strategy_update: SMACrossoverStrategyActiveModel = strategy.into();
    let mut updated = false;

    if let Some(ref name) = body.name {
        strategy_update.name = Set(name.clone());
        updated = true;
    }

    if let Some(ref status) = body.status {
        strategy_update.status = Set(status.clone().into());
        updated = true;
    }

    if let Some(ref config) = body.config {
        // Validate new config
        config.validate().map_err(|e| AppError::BadRequest(format!("Invalid SMACrossoverConfig: {}", e)))?;

        let config_json = serde_json::to_string(config)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize config: {}", e)))?;

        strategy_update.config_json = Set(config_json);
        updated = true;
    }

    if updated {
        strategy_update.updated_at = Set(Utc::now());
        let updated_strategy = strategy_update.update(db.get_ref()).await
            .map_err(AppError::DatabaseError)?;

        let config = updated_strategy.get_sma_crossover_config().map_err(|e| AppError::BadRequest(e))?;

        let response = SMACrossoverStrategyResponse {
            id: updated_strategy.id,
            user_id: updated_strategy.user_id,
            name: updated_strategy.name.clone(),
            asset_symbol: updated_strategy.asset_symbol.clone(),
            status: updated_strategy.status.clone(),
            config,
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
            last_fast_sma: updated_strategy.last_fast_sma,
            last_slow_sma: updated_strategy.last_slow_sma,
            sma_spread: updated_strategy.calculate_sma_spread(),
            last_signal_type: updated_strategy.last_signal_type,
            last_signal_time: updated_strategy.last_signal_time,
            last_execution_at: updated_strategy.last_execution_at,
            recent_executions: vec![], // Would need separate query for this
            created_at: updated_strategy.created_at,
            updated_at: updated_strategy.updated_at,
        };

        Ok(HttpResponse::Ok().json(response))
    } else {
        Err(AppError::BadRequest("No fields to update".to_string()))
    }
}

/// Delete an SMA Crossover strategy
pub async fn delete_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Find and validate ownership
    let strategy = SMACrossoverStrategyEntity::find_by_id(strategy_id)
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Delete associated executions first
    SMACrossoverExecutionEntity::delete_many()
        .filter(crate::models::sma_crossover_strategy::execution::Column::StrategyId.eq(strategy_id))
        .exec(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Delete the strategy
    let strategy_model: SMACrossoverStrategyActiveModel = strategy.into();
    strategy_model.delete(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::NoContent().finish())
}

/// Pause an SMA Crossover strategy
pub async fn pause_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Find and validate ownership
    let strategy = SMACrossoverStrategyEntity::find_by_id(strategy_id)
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    if strategy.status == "paused" {
        return Err(AppError::BadRequest("Strategy is already paused".to_string()));
    }

    let mut strategy_update: SMACrossoverStrategyActiveModel = strategy.into();
    strategy_update.status = Set("paused".to_string());
    strategy_update.updated_at = Set(Utc::now());

    strategy_update.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy paused successfully"
    })))
}

/// Resume an SMA Crossover strategy
pub async fn resume_sma_crossover_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Find and validate ownership
    let strategy = SMACrossoverStrategyEntity::find_by_id(strategy_id)
        .filter(crate::models::sma_crossover_strategy::Column::UserId.eq(user_id))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    if strategy.status != "paused" {
        return Err(AppError::BadRequest("Strategy is not paused".to_string()));
    }

    let mut strategy_update: SMACrossoverStrategyActiveModel = strategy.into();
    strategy_update.status = Set("active".to_string());
    strategy_update.updated_at = Set(Utc::now());

    strategy_update.update(db.get_ref()).await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy resumed successfully"
    })))
}
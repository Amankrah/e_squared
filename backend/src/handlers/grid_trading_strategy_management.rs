use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use std::sync::Arc;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;

use crate::models::grid_trading_strategy::{
    Entity as GridTradingStrategyEntity,
    ActiveModel as GridTradingStrategyActiveModel,
    ExecutionEntity as GridTradingExecutionEntity,
    CreateGridTradingStrategyRequest, UpdateGridTradingStrategyRequest,
    GridTradingStrategyResponse, GridTradingStrategiesResponse, GridTradingExecutionResponse,
    GridTradingStatus,
};
use crate::services::MarketDataService;
use crate::utils::errors::AppError;
use actix_session::SessionExt;

/// Extract authenticated user ID from session
fn get_user_id_from_session(req: &HttpRequest) -> Result<Uuid, AppError> {
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

    Err(AppError::Unauthorized("Authentication required".to_string()))
}

/// Create a new Grid Trading strategy
pub async fn create_grid_trading_strategy(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    body: web::Json<CreateGridTradingStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from session
    let user_id = get_user_id_from_session(&req)?;

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Check if user already has a strategy with this name
    let existing_strategy = GridTradingStrategyEntity::find()
        .filter(crate::models::grid_trading_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::grid_trading_strategy::Column::Name.eq(&body.name))
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_strategy.is_some() {
        return Err(AppError::BadRequest("Strategy with this name already exists".to_string()));
    }

    // Validate GridTradingConfig
    body.config.validate().map_err(|e| AppError::BadRequest(format!("Invalid GridTradingConfig: {}", e)))?;

    // Serialize GridTradingConfig to JSON
    let config_json = serde_json::to_string(&body.config)
        .map_err(|e| AppError::BadRequest(format!("Failed to serialize GridTradingConfig: {}", e)))?;

    // Create the strategy
    let strategy_id = Uuid::new_v4();
    let new_strategy = GridTradingStrategyActiveModel {
        id: Set(strategy_id),
        user_id: Set(user_id),
        name: Set(body.name.clone()),
        asset_symbol: Set(body.asset_symbol.to_uppercase()),
        status: Set(GridTradingStatus::Active.into()),
        config_json: Set(config_json),
        total_invested: Set(Decimal::ZERO),
        total_purchased: Set(Decimal::ZERO),
        average_buy_price: Set(None),
        current_inventory: Set(Decimal::ZERO),
        grid_levels_count: Set(body.config.grid_levels as i32),
        total_trades: Set(0),
        winning_trades: Set(0),
        losing_trades: Set(0),
        realized_pnl: Set(Decimal::ZERO),
        unrealized_pnl: Set(None),
        max_drawdown: Set(None),
        grid_center_price: Set(None),
        grid_upper_bound: Set(None),
        grid_lower_bound: Set(None),
        last_rebalance_at: Set(None),
        total_grid_profit: Set(Decimal::ZERO),
        active_buy_orders: Set(0),
        active_sell_orders: Set(0),
        last_execution_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    // Insert without returning (to avoid UnpackInsertId error)
    GridTradingStrategyEntity::insert(new_strategy)
        .exec_without_returning(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Fetch the created strategy
    let strategy = GridTradingStrategyEntity::find_by_id(strategy_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or(AppError::InternalServerError)?;

    // Convert to response format
    let response = GridTradingStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_grid_trading_config().map_err(|e| AppError::BadRequest(e))?,
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_inventory: strategy.current_inventory,
        grid_levels_count: strategy.grid_levels_count,
        total_trades: strategy.total_trades,
        winning_trades: strategy.winning_trades,
        losing_trades: strategy.losing_trades,
        win_rate: strategy.calculate_win_rate(),
        realized_pnl: strategy.realized_pnl,
        unrealized_pnl: strategy.unrealized_pnl,
        total_pnl: strategy.calculate_total_pnl(),
        max_drawdown: strategy.max_drawdown,
        grid_center_price: strategy.grid_center_price,
        grid_upper_bound: strategy.grid_upper_bound,
        grid_lower_bound: strategy.grid_lower_bound,
        grid_spread: strategy.calculate_grid_spread(),
        last_rebalance_at: strategy.last_rebalance_at,
        total_grid_profit: strategy.total_grid_profit,
        active_buy_orders: strategy.active_buy_orders,
        active_sell_orders: strategy.active_sell_orders,
        grid_utilization: strategy.calculate_grid_utilization(),
        inventory_utilization: strategy.calculate_inventory_utilization(),
        last_execution_at: strategy.last_execution_at,
        recent_executions: Vec::new(),
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Get user's Grid Trading strategies
pub async fn get_grid_trading_strategies(
    db: web::Data<Arc<DatabaseConnection>>,
    market_service: web::Data<MarketDataService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    // Get user's strategies
    let strategies = GridTradingStrategyEntity::find()
        .filter(crate::models::grid_trading_strategy::Column::UserId.eq(user_id))
        .order_by_desc(crate::models::grid_trading_strategy::Column::CreatedAt)
        .all(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut strategy_responses = Vec::new();
    let mut total_invested = Decimal::ZERO;
    let mut total_pnl = Decimal::ZERO;
    let mut active_strategies = 0;
    let mut total_win_rate = Decimal::ZERO;
    let mut total_grid_profit = Decimal::ZERO;

    for strategy in strategies {
        // Get recent executions for this strategy
        let recent_executions = GridTradingExecutionEntity::find()
            .filter(crate::models::grid_trading_strategy::execution::Column::StrategyId.eq(strategy.id))
            .order_by_desc(crate::models::grid_trading_strategy::execution::Column::ExecutionTimestamp)
            .limit(10)
            .all(db.as_ref().as_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        let execution_responses: Vec<GridTradingExecutionResponse> = recent_executions.into_iter()
            .map(|exec| GridTradingExecutionResponse {
                id: exec.id,
                strategy_id: exec.strategy_id,
                execution_type: exec.execution_type,
                trigger_reason: exec.trigger_reason,
                amount_usd: exec.amount_usd,
                amount_asset: exec.amount_asset,
                price_at_execution: exec.price_at_execution,
                grid_level_index: exec.grid_level_index,
                grid_level_price: exec.grid_level_price,
                inventory_before: exec.inventory_before,
                inventory_after: exec.inventory_after,
                grid_profit: exec.grid_profit,
                order_status: exec.order_status,
                execution_timestamp: exec.execution_timestamp,
                error_message: exec.error_message,
            })
            .collect();

        // Calculate current unrealized P&L if we have inventory
        let unrealized_pnl = if strategy.current_inventory != Decimal::ZERO {
            match market_service.get_current_price(&strategy.asset_symbol).await {
                Ok(current_price) => {
                    if let Some(avg_price) = strategy.average_buy_price {
                        let current_value = strategy.current_inventory.abs() * current_price;
                        let invested_value = strategy.current_inventory.abs() * avg_price;
                        Some(if strategy.current_inventory > Decimal::ZERO {
                            current_value - invested_value // Long position
                        } else {
                            invested_value - current_value // Short position
                        })
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
        total_grid_profit += strategy.total_grid_profit;
        if strategy.status == String::from(GridTradingStatus::Active) {
            active_strategies += 1;
        }
        total_win_rate += strategy.calculate_win_rate();

        let strategy_response = GridTradingStrategyResponse {
            id: strategy.id,
            user_id: strategy.user_id,
            name: strategy.name.clone(),
            asset_symbol: strategy.asset_symbol.clone(),
            status: strategy.status.clone(),
            config: strategy.get_grid_trading_config().unwrap_or_else(|_| Default::default()),
            total_invested: strategy.total_invested,
            total_purchased: strategy.total_purchased,
            average_buy_price: strategy.average_buy_price,
            current_inventory: strategy.current_inventory,
            grid_levels_count: strategy.grid_levels_count,
            total_trades: strategy.total_trades,
            winning_trades: strategy.winning_trades,
            losing_trades: strategy.losing_trades,
            win_rate: strategy.calculate_win_rate(),
            realized_pnl: strategy.realized_pnl,
            unrealized_pnl,
            total_pnl: strategy.calculate_total_pnl(),
            max_drawdown: strategy.max_drawdown,
            grid_center_price: strategy.grid_center_price,
            grid_upper_bound: strategy.grid_upper_bound,
            grid_lower_bound: strategy.grid_lower_bound,
            grid_spread: strategy.calculate_grid_spread(),
            last_rebalance_at: strategy.last_rebalance_at,
            total_grid_profit: strategy.total_grid_profit,
            active_buy_orders: strategy.active_buy_orders,
            active_sell_orders: strategy.active_sell_orders,
            grid_utilization: strategy.calculate_grid_utilization(),
            inventory_utilization: strategy.calculate_inventory_utilization(),
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

    let response = GridTradingStrategiesResponse {
        strategies: strategy_responses,
        total_invested,
        total_pnl,
        active_strategies,
        average_win_rate,
        total_grid_profit,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get a specific Grid Trading strategy
pub async fn get_grid_trading_strategy(
    db: web::Data<Arc<DatabaseConnection>>,
    market_service: web::Data<MarketDataService>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    let strategy_id = path.into_inner();

    // Get the strategy
    let strategy = GridTradingStrategyEntity::find_by_id(strategy_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Get recent executions
    let recent_executions = GridTradingExecutionEntity::find()
        .filter(crate::models::grid_trading_strategy::execution::Column::StrategyId.eq(strategy.id))
        .order_by_desc(crate::models::grid_trading_strategy::execution::Column::ExecutionTimestamp)
        .limit(50)
        .all(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let execution_responses: Vec<GridTradingExecutionResponse> = recent_executions.into_iter()
        .map(|exec| GridTradingExecutionResponse {
            id: exec.id,
            strategy_id: exec.strategy_id,
            execution_type: exec.execution_type,
            trigger_reason: exec.trigger_reason,
            amount_usd: exec.amount_usd,
            amount_asset: exec.amount_asset,
            price_at_execution: exec.price_at_execution,
            grid_level_index: exec.grid_level_index,
            grid_level_price: exec.grid_level_price,
            inventory_before: exec.inventory_before,
            inventory_after: exec.inventory_after,
            grid_profit: exec.grid_profit,
            order_status: exec.order_status,
            execution_timestamp: exec.execution_timestamp,
            error_message: exec.error_message,
        })
        .collect();

    // Calculate current unrealized P&L
    let unrealized_pnl = if strategy.current_inventory != Decimal::ZERO {
        match market_service.get_current_price(&strategy.asset_symbol).await {
            Ok(current_price) => {
                if let Some(avg_price) = strategy.average_buy_price {
                    let current_value = strategy.current_inventory.abs() * current_price;
                    let invested_value = strategy.current_inventory.abs() * avg_price;
                    Some(if strategy.current_inventory > Decimal::ZERO {
                        current_value - invested_value // Long position
                    } else {
                        invested_value - current_value // Short position
                    })
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    let response = GridTradingStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_grid_trading_config().unwrap_or_else(|_| Default::default()),
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_inventory: strategy.current_inventory,
        grid_levels_count: strategy.grid_levels_count,
        total_trades: strategy.total_trades,
        winning_trades: strategy.winning_trades,
        losing_trades: strategy.losing_trades,
        win_rate: strategy.calculate_win_rate(),
        realized_pnl: strategy.realized_pnl,
        unrealized_pnl,
        total_pnl: strategy.calculate_total_pnl(),
        max_drawdown: strategy.max_drawdown,
        grid_center_price: strategy.grid_center_price,
        grid_upper_bound: strategy.grid_upper_bound,
        grid_lower_bound: strategy.grid_lower_bound,
        grid_spread: strategy.calculate_grid_spread(),
        last_rebalance_at: strategy.last_rebalance_at,
        total_grid_profit: strategy.total_grid_profit,
        active_buy_orders: strategy.active_buy_orders,
        active_sell_orders: strategy.active_sell_orders,
        grid_utilization: strategy.calculate_grid_utilization(),
        inventory_utilization: strategy.calculate_inventory_utilization(),
        last_execution_at: strategy.last_execution_at,
        recent_executions: execution_responses,
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Update a Grid Trading strategy
pub async fn update_grid_trading_strategy(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateGridTradingStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    let strategy_id = path.into_inner();

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Get the strategy
    let mut strategy: GridTradingStrategyActiveModel = GridTradingStrategyEntity::find_by_id(strategy_id)
        .one(db.as_ref().as_ref())
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
        config.validate().map_err(|e| AppError::BadRequest(format!("Invalid GridTradingConfig: {}", e)))?;

        // Serialize the new config to JSON
        let config_json = serde_json::to_string(&config)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize GridTradingConfig: {}", e)))?;

        strategy.config_json = Set(config_json);
        strategy.grid_levels_count = Set(config.grid_levels as i32);
    }

    strategy.updated_at = Set(Utc::now());

    // Save changes
    let updated_strategy = strategy.update(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = GridTradingStrategyResponse {
        id: updated_strategy.id,
        user_id: updated_strategy.user_id,
        name: updated_strategy.name.clone(),
        asset_symbol: updated_strategy.asset_symbol.clone(),
        status: updated_strategy.status.clone(),
        config: updated_strategy.get_grid_trading_config().unwrap_or_else(|_| Default::default()),
        total_invested: updated_strategy.total_invested,
        total_purchased: updated_strategy.total_purchased,
        average_buy_price: updated_strategy.average_buy_price,
        current_inventory: updated_strategy.current_inventory,
        grid_levels_count: updated_strategy.grid_levels_count,
        total_trades: updated_strategy.total_trades,
        winning_trades: updated_strategy.winning_trades,
        losing_trades: updated_strategy.losing_trades,
        win_rate: updated_strategy.calculate_win_rate(),
        realized_pnl: updated_strategy.realized_pnl,
        unrealized_pnl: updated_strategy.unrealized_pnl,
        total_pnl: updated_strategy.calculate_total_pnl(),
        max_drawdown: updated_strategy.max_drawdown,
        grid_center_price: updated_strategy.grid_center_price,
        grid_upper_bound: updated_strategy.grid_upper_bound,
        grid_lower_bound: updated_strategy.grid_lower_bound,
        grid_spread: updated_strategy.calculate_grid_spread(),
        last_rebalance_at: updated_strategy.last_rebalance_at,
        total_grid_profit: updated_strategy.total_grid_profit,
        active_buy_orders: updated_strategy.active_buy_orders,
        active_sell_orders: updated_strategy.active_sell_orders,
        grid_utilization: updated_strategy.calculate_grid_utilization(),
        inventory_utilization: updated_strategy.calculate_inventory_utilization(),
        last_execution_at: updated_strategy.last_execution_at,
        recent_executions: Vec::new(),
        created_at: updated_strategy.created_at,
        updated_at: updated_strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete a Grid Trading strategy
pub async fn delete_grid_trading_strategy(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
    path: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    let strategy_id = path.into_inner();

    // Get the strategy
    let strategy = GridTradingStrategyEntity::find_by_id(strategy_id)
        .one(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Delete the strategy (cascading delete will handle executions)
    GridTradingStrategyEntity::delete_by_id(strategy_id)
        .exec(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy deleted successfully"
    })))
}

/// Get Grid Trading strategy execution statistics
pub async fn get_grid_trading_execution_stats(
    db: web::Data<Arc<DatabaseConnection>>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = get_user_id_from_session(&req)?;

    // Get user's strategies to calculate stats
    let strategies = GridTradingStrategyEntity::find()
        .filter(crate::models::grid_trading_strategy::Column::UserId.eq(user_id))
        .all(db.as_ref().as_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut total_trades = 0;
    let mut total_winning_trades = 0;
    let mut total_invested = Decimal::ZERO;
    let mut total_pnl = Decimal::ZERO;
    let mut total_grid_profit = Decimal::ZERO;
    let mut total_inventory = Decimal::ZERO;

    for strategy in strategies {
        total_trades += strategy.total_trades;
        total_winning_trades += strategy.winning_trades;
        total_invested += strategy.total_invested;
        total_grid_profit += strategy.total_grid_profit;
        total_inventory += strategy.current_inventory.abs();
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
        "total_grid_profit": total_grid_profit,
        "current_inventory": total_inventory,
        "roi_percentage": if total_invested > Decimal::ZERO {
            (total_pnl / total_invested) * Decimal::from(100)
        } else {
            Decimal::ZERO
        },
        "grid_profit_percentage": if total_invested > Decimal::ZERO {
            (total_grid_profit / total_invested) * Decimal::from(100)
        } else {
            Decimal::ZERO
        }
    });

    Ok(HttpResponse::Ok().json(stats))
}

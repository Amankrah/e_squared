use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, QueryOrder, QuerySelect};
use uuid::Uuid;
use validator::Validate;
use rust_decimal::Decimal;

use crate::models::dca_strategy::{
    Entity as DCAStrategyEntity,
    ActiveModel as DCAStrategyActiveModel,
    ExecutionEntity as DCAExecutionEntity,
    CreateDCAStrategyRequest, UpdateDCAStrategyRequest,
    DCAStrategyResponse, DCAStrategiesResponse, DCAExecutionResponse,
    DCAStrategyType, DCAStatus,
};
use crate::services::{DCAExecutionEngine, MarketDataService};
use crate::utils::errors::AppError;

/// Create a new DCA strategy
pub async fn create_dca_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    body: web::Json<CreateDCAStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user ID from request extensions (set by auth middleware)
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Check if user already has a strategy with this name
    let existing_strategy = DCAStrategyEntity::find()
        .filter(crate::models::dca_strategy::Column::UserId.eq(user_id))
        .filter(crate::models::dca_strategy::Column::Name.eq(&body.name))
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    if existing_strategy.is_some() {
        return Err(AppError::BadRequest("Strategy with this name already exists".to_string()));
    }

    // Validate DCAConfig
    body.config.validate().map_err(|e| AppError::BadRequest(format!("Invalid DCAConfig: {}", e)))?;

    // Serialize DCAConfig to JSON
    let config_json = serde_json::to_string(&body.config)
        .map_err(|e| AppError::BadRequest(format!("Failed to serialize DCAConfig: {}", e)))?;

    // Calculate initial next execution time based on strategy frequency
    use crate::strategies::implementations::dca::DCAFrequency;
    let next_execution_at = match body.config.frequency {
        DCAFrequency::Hourly(hours) => Utc::now() + chrono::Duration::hours(hours as i64),
        DCAFrequency::Daily(days) => Utc::now() + chrono::Duration::days(days as i64),
        DCAFrequency::Weekly(weeks) => Utc::now() + chrono::Duration::weeks(weeks as i64),
        DCAFrequency::Monthly(months) => Utc::now() + chrono::Duration::days((months * 30) as i64),
        DCAFrequency::Custom(minutes) => Utc::now() + chrono::Duration::minutes(minutes as i64),
    };

    // Create the strategy
    let new_strategy = DCAStrategyActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set(body.name.clone()),
        asset_symbol: Set(body.asset_symbol.to_uppercase()),
        status: Set(DCAStatus::Active.into()),
        config_json: Set(config_json),
        total_invested: Set(Decimal::ZERO),
        total_purchased: Set(Decimal::ZERO),
        average_buy_price: Set(None),
        last_execution_at: Set(None),
        next_execution_at: Set(Some(next_execution_at)),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let strategy = new_strategy.insert(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = DCAStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name,
        asset_symbol: strategy.asset_symbol,
        total_allocation: strategy.total_allocation,
        base_tranche_size: strategy.base_tranche_size,
        status: strategy.status,
        strategy_type: strategy.strategy_type,
        sentiment_multiplier: strategy.sentiment_multiplier,
        volatility_adjustment: strategy.volatility_adjustment,
        fear_greed_threshold_buy: strategy.fear_greed_threshold_buy,
        fear_greed_threshold_sell: strategy.fear_greed_threshold_sell,
        max_tranche_percentage: strategy.max_tranche_percentage,
        min_tranche_percentage: strategy.min_tranche_percentage,
        dca_interval_hours: strategy.dca_interval_hours,
        target_zones: strategy.target_zones.as_ref()
            .and_then(|zones_str| serde_json::from_str::<Vec<Decimal>>(zones_str).ok()),
        stop_loss_percentage: strategy.stop_loss_percentage,
        take_profit_percentage: strategy.take_profit_percentage,
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_profit_loss: None,
        profit_loss_percentage: None,
        last_execution_at: strategy.last_execution_at,
        next_execution_at: strategy.next_execution_at,
        recent_executions: Vec::new(),
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Get user's DCA strategies
pub async fn get_dca_strategies(
    db: web::Data<DatabaseConnection>,
    market_service: web::Data<MarketDataService>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    // Get user's strategies
    let strategies = DCAStrategyEntity::find()
        .filter(crate::models::dca_strategy::Column::UserId.eq(user_id))
        .order_by_desc(crate::models::dca_strategy::Column::CreatedAt)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let mut strategy_responses = Vec::new();
    let mut total_allocation = Decimal::ZERO;
    let mut total_invested = Decimal::ZERO;
    let mut total_profit_loss = Decimal::ZERO;
    let mut active_strategies = 0;

    for strategy in strategies {
        // Get recent executions for this strategy
        let recent_executions = DCAExecutionEntity::find()
            .filter(crate::models::dca_strategy::execution::Column::StrategyId.eq(strategy.id))
            .order_by_desc(crate::models::dca_strategy::execution::Column::ExecutionTimestamp)
            .limit(10)
            .all(db.get_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        let execution_responses: Vec<DCAExecutionResponse> = recent_executions.into_iter()
            .map(|exec| DCAExecutionResponse {
                id: exec.id,
                strategy_id: exec.strategy_id,
                execution_type: exec.execution_type,
                trigger_reason: exec.trigger_reason,
                amount_usd: exec.amount_usd,
                amount_asset: exec.amount_asset,
                price_at_execution: exec.price_at_execution,
                fear_greed_index: exec.fear_greed_index,
                market_volatility: exec.market_volatility,
                order_status: exec.order_status,
                execution_timestamp: exec.execution_timestamp,
                error_message: exec.error_message,
            })
            .collect();

        // Calculate current P&L if we have positions
        let (current_profit_loss, profit_loss_percentage) = if strategy.total_purchased > Decimal::ZERO {
            match market_service.get_current_price(&strategy.asset_symbol).await {
                Ok(current_price) => {
                    if let Some(avg_price) = strategy.average_buy_price {
                        let current_value = strategy.total_purchased * current_price;
                        let invested_value = strategy.total_purchased * avg_price;
                        let profit_loss = current_value - invested_value;
                        let profit_loss_pct = if invested_value > Decimal::ZERO {
                            (profit_loss / invested_value) * Decimal::from(100)
                        } else {
                            Decimal::ZERO
                        };
                        (Some(profit_loss), Some(profit_loss_pct))
                    } else {
                        (None, None)
                    }
                }
                Err(_) => (None, None),
            }
        } else {
            (None, None)
        };

        // Update totals
        total_allocation += strategy.total_allocation;
        total_invested += strategy.total_invested;
        if let Some(pnl) = current_profit_loss {
            total_profit_loss += pnl;
        }
        if strategy.status == String::from(DCAStatus::Active) {
            active_strategies += 1;
        }

        let strategy_response = DCAStrategyResponse {
            id: strategy.id,
            user_id: strategy.user_id,
            name: strategy.name,
            asset_symbol: strategy.asset_symbol,
            total_allocation: strategy.total_allocation,
            base_tranche_size: strategy.base_tranche_size,
            status: strategy.status,
            strategy_type: strategy.strategy_type,
            sentiment_multiplier: strategy.sentiment_multiplier,
            volatility_adjustment: strategy.volatility_adjustment,
            fear_greed_threshold_buy: strategy.fear_greed_threshold_buy,
            fear_greed_threshold_sell: strategy.fear_greed_threshold_sell,
            max_tranche_percentage: strategy.max_tranche_percentage,
            min_tranche_percentage: strategy.min_tranche_percentage,
            dca_interval_hours: strategy.dca_interval_hours,
            target_zones: strategy.target_zones.as_ref()
                .and_then(|zones_str| serde_json::from_str::<Vec<Decimal>>(zones_str).ok()),
            stop_loss_percentage: strategy.stop_loss_percentage,
            take_profit_percentage: strategy.take_profit_percentage,
            total_invested: strategy.total_invested,
            total_purchased: strategy.total_purchased,
            average_buy_price: strategy.average_buy_price,
            current_profit_loss,
            profit_loss_percentage,
            last_execution_at: strategy.last_execution_at,
            next_execution_at: strategy.next_execution_at,
            recent_executions: execution_responses,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        };

        strategy_responses.push(strategy_response);
    }

    let response = DCAStrategiesResponse {
        strategies: strategy_responses,
        total_allocation,
        total_invested,
        total_profit_loss,
        active_strategies,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get a specific DCA strategy
pub async fn get_dca_strategy(
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
    let strategy = DCAStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Get recent executions
    let recent_executions = DCAExecutionEntity::find()
        .filter(crate::models::dca_strategy::execution::Column::StrategyId.eq(strategy.id))
        .order_by_desc(crate::models::dca_strategy::execution::Column::ExecutionTimestamp)
        .limit(50)
        .all(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    let execution_responses: Vec<DCAExecutionResponse> = recent_executions.into_iter()
        .map(|exec| DCAExecutionResponse {
            id: exec.id,
            strategy_id: exec.strategy_id,
            execution_type: exec.execution_type,
            trigger_reason: exec.trigger_reason,
            amount_usd: exec.amount_usd,
            amount_asset: exec.amount_asset,
            price_at_execution: exec.price_at_execution,
            fear_greed_index: exec.fear_greed_index,
            market_volatility: exec.market_volatility,
            order_status: exec.order_status,
            execution_timestamp: exec.execution_timestamp,
            error_message: exec.error_message,
        })
        .collect();

    // Calculate current P&L
    let (current_profit_loss, profit_loss_percentage) = if strategy.total_purchased > Decimal::ZERO {
        match market_service.get_current_price(&strategy.asset_symbol).await {
            Ok(current_price) => {
                if let Some(avg_price) = strategy.average_buy_price {
                    let current_value = strategy.total_purchased * current_price;
                    let invested_value = strategy.total_purchased * avg_price;
                    let profit_loss = current_value - invested_value;
                    let profit_loss_pct = if invested_value > Decimal::ZERO {
                        (profit_loss / invested_value) * Decimal::from(100)
                    } else {
                        Decimal::ZERO
                    };
                    (Some(profit_loss), Some(profit_loss_pct))
                } else {
                    (None, None)
                }
            }
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    let response = DCAStrategyResponse {
        id: strategy.id,
        user_id: strategy.user_id,
        name: strategy.name,
        asset_symbol: strategy.asset_symbol,
        total_allocation: strategy.total_allocation,
        base_tranche_size: strategy.base_tranche_size,
        status: strategy.status,
        strategy_type: strategy.strategy_type,
        sentiment_multiplier: strategy.sentiment_multiplier,
        volatility_adjustment: strategy.volatility_adjustment,
        fear_greed_threshold_buy: strategy.fear_greed_threshold_buy,
        fear_greed_threshold_sell: strategy.fear_greed_threshold_sell,
        max_tranche_percentage: strategy.max_tranche_percentage,
        min_tranche_percentage: strategy.min_tranche_percentage,
        dca_interval_hours: strategy.dca_interval_hours,
        target_zones: strategy.target_zones.as_ref()
            .and_then(|zones_str| serde_json::from_str::<Vec<Decimal>>(zones_str).ok()),
        stop_loss_percentage: strategy.stop_loss_percentage,
        take_profit_percentage: strategy.take_profit_percentage,
        total_invested: strategy.total_invested,
        total_purchased: strategy.total_purchased,
        average_buy_price: strategy.average_buy_price,
        current_profit_loss,
        profit_loss_percentage,
        last_execution_at: strategy.last_execution_at,
        next_execution_at: strategy.next_execution_at,
        recent_executions: execution_responses,
        created_at: strategy.created_at,
        updated_at: strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Update a DCA strategy
pub async fn update_dca_strategy(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<UpdateDCAStrategyRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Validate request
    body.validate().map_err(AppError::ValidationError)?;

    // Get the strategy
    let mut strategy: DCAStrategyActiveModel = DCAStrategyEntity::find_by_id(strategy_id)
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

    if let Some(total_allocation) = body.total_allocation {
        strategy.total_allocation = Set(total_allocation);

        // Recalculate base tranche size if allocation changes
        if let Some(base_percentage) = body.base_tranche_percentage {
            let new_tranche_size = total_allocation * base_percentage / Decimal::from(100);
            strategy.base_tranche_size = Set(new_tranche_size);
        }
    }

    if let Some(status) = &body.status {
        strategy.status = Set(status.clone().into());
    }

    if let Some(sentiment_multiplier) = body.sentiment_multiplier {
        strategy.sentiment_multiplier = Set(sentiment_multiplier);
    }

    if let Some(volatility_adjustment) = body.volatility_adjustment {
        strategy.volatility_adjustment = Set(volatility_adjustment);
    }

    if let Some(fear_greed_buy) = body.fear_greed_threshold_buy {
        strategy.fear_greed_threshold_buy = Set(fear_greed_buy);
    }

    if let Some(fear_greed_sell) = body.fear_greed_threshold_sell {
        strategy.fear_greed_threshold_sell = Set(fear_greed_sell);
    }

    if let Some(interval_hours) = body.dca_interval_hours {
        strategy.dca_interval_hours = Set(interval_hours);
    }

    if let Some(zones) = &body.target_zones {
        strategy.target_zones = Set(Some(serde_json::to_string(zones).unwrap()));
    }

    if let Some(stop_loss) = body.stop_loss_percentage {
        strategy.stop_loss_percentage = Set(Some(stop_loss));
    }

    if let Some(take_profit) = body.take_profit_percentage {
        strategy.take_profit_percentage = Set(Some(take_profit));
    }

    strategy.updated_at = Set(Utc::now());

    // Save changes
    let updated_strategy = strategy.update(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    // Convert to response format
    let response = DCAStrategyResponse {
        id: updated_strategy.id,
        user_id: updated_strategy.user_id,
        name: updated_strategy.name,
        asset_symbol: updated_strategy.asset_symbol,
        total_allocation: updated_strategy.total_allocation,
        base_tranche_size: updated_strategy.base_tranche_size,
        status: updated_strategy.status,
        strategy_type: updated_strategy.strategy_type,
        sentiment_multiplier: updated_strategy.sentiment_multiplier,
        volatility_adjustment: updated_strategy.volatility_adjustment,
        fear_greed_threshold_buy: updated_strategy.fear_greed_threshold_buy,
        fear_greed_threshold_sell: updated_strategy.fear_greed_threshold_sell,
        max_tranche_percentage: updated_strategy.max_tranche_percentage,
        min_tranche_percentage: updated_strategy.min_tranche_percentage,
        dca_interval_hours: updated_strategy.dca_interval_hours,
        target_zones: updated_strategy.target_zones.as_ref()
            .and_then(|zones_str| serde_json::from_str::<Vec<Decimal>>(zones_str).ok()),
        stop_loss_percentage: updated_strategy.stop_loss_percentage,
        take_profit_percentage: updated_strategy.take_profit_percentage,
        total_invested: updated_strategy.total_invested,
        total_purchased: updated_strategy.total_purchased,
        average_buy_price: updated_strategy.average_buy_price,
        current_profit_loss: None,
        profit_loss_percentage: None,
        last_execution_at: updated_strategy.last_execution_at,
        next_execution_at: updated_strategy.next_execution_at,
        recent_executions: Vec::new(),
        created_at: updated_strategy.created_at,
        updated_at: updated_strategy.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete a DCA strategy
pub async fn delete_dca_strategy(
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
    let strategy = DCAStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    // Verify ownership
    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Delete the strategy (cascading delete will handle executions)
    DCAStrategyEntity::delete_by_id(strategy_id)
        .exec(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy deleted successfully"
    })))
}

/// Manually execute a DCA strategy
pub async fn execute_dca_strategy(
    db: web::Data<DatabaseConnection>,
    execution_engine: web::Data<DCAExecutionEngine>,
    req: HttpRequest,
    path: web::Path<Uuid>,
    body: web::Json<serde_json::Value>,
) -> Result<HttpResponse, AppError> {
    let user_id = req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let strategy_id = path.into_inner();

    // Verify ownership before allowing manual execution
    let strategy = DCAStrategyEntity::find_by_id(strategy_id)
        .one(db.get_ref())
        .await
        .map_err(AppError::DatabaseError)?
        .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

    if strategy.user_id != user_id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    // Get optional manual amount from request body
    let manual_amount = body.get("amount_usd")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<Decimal>().ok());

    // Queue the strategy for manual execution
    execution_engine.queue_manual_execution(strategy_id, manual_amount).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Strategy queued for execution",
        "strategy_id": strategy_id,
        "manual_amount": manual_amount
    })))
}

/// Get DCA execution statistics
pub async fn get_execution_stats(
    execution_engine: web::Data<DCAExecutionEngine>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    // Verify authentication (user_id not used for global stats, but auth is required)
    req.extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;

    let stats = execution_engine.get_execution_stats().await;

    Ok(HttpResponse::Ok().json(stats))
}
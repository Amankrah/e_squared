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
    DCAStatus,
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
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_dca_config().map_err(|e| AppError::BadRequest(e))?,
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
        let config = strategy.get_dca_config().unwrap_or_else(|_| Default::default());
        total_allocation += config.base_amount * Decimal::from(12); // Assume 12 months for total allocation
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
            status: strategy.status,
            config: config,
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
        name: strategy.name.clone(),
        asset_symbol: strategy.asset_symbol.clone(),
        status: strategy.status.clone(),
        config: strategy.get_dca_config().unwrap_or_else(|_| Default::default()),
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

    // Update status if provided
    if let Some(status) = &body.status {
        strategy.status = Set(status.clone().into());
    }

    // Update config if provided
    if let Some(config) = &body.config {
        // Validate the new config
        config.validate().map_err(|e| AppError::BadRequest(format!("Invalid DCAConfig: {}", e)))?;

        // Serialize the new config to JSON
        let config_json = serde_json::to_string(&config)
            .map_err(|e| AppError::BadRequest(format!("Failed to serialize DCAConfig: {}", e)))?;

        strategy.config_json = Set(config_json);
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
        name: updated_strategy.name.clone(),
        asset_symbol: updated_strategy.asset_symbol.clone(),
        status: updated_strategy.status.clone(),
        config: updated_strategy.get_dca_config().unwrap_or_else(|_| Default::default()),
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
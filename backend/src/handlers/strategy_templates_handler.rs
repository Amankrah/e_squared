use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use tracing::{info, error};

use crate::services::strategy_templates::{StrategyTemplateService, StrategyTemplate};
use crate::backtesting::{BacktestEngine, BacktestConfig, BacktestResult};
use crate::strategies::{StrategyRegistry, StrategyInfo};
use crate::exchange_connectors::KlineInterval;
use crate::utils::errors::AppError;
use crate::handlers::auth::Claims;
use crate::models::dca_strategy::DCAStrategyType;

#[derive(Debug, Deserialize)]
pub struct TemplateQuery {
    pub category: Option<String>,
    pub risk_level: Option<String>,
    pub complexity: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BacktestTemplateRequest {
    pub template_id: String,
    pub symbol: String,
    pub interval: KlineInterval,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub initial_balance: Decimal,
    pub template_parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct TemplateBacktestResponse {
    pub template_id: String,
    pub template_name: String,
    pub backtest_result: BacktestResult,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
}

// StrategyTemplate is now imported from services::strategy_templates

#[derive(Debug, Serialize)]
pub struct TemplatesResponse {
    pub templates: Vec<StrategyTemplate>,
    pub total_count: usize,
}

/// Get all strategy templates with optional filtering
pub async fn get_strategy_templates(
    template_service: web::Data<StrategyTemplateService>,
    query: web::Query<TemplateQuery>,
) -> Result<HttpResponse, AppError> {
    info!("Fetching strategy templates with filters: {:?}", query);

    // Use the actual template service
    let mut templates = template_service.get_all_templates()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    // Apply filters
    if let Some(ref category) = query.category {
        templates.retain(|t| t.category.to_string().to_lowercase() == category.to_lowercase());
    }

    if let Some(ref risk_level) = query.risk_level {
        templates.retain(|t| t.risk_level.to_string().to_lowercase() == risk_level.to_lowercase());
    }

    if let Some(ref complexity) = query.complexity {
        templates.retain(|t| t.complexity.to_string().to_lowercase() == complexity.to_lowercase());
    }

    Ok(HttpResponse::Ok().json(TemplatesResponse {
        total_count: templates.len(),
        templates,
    }))
}

/// Get a specific strategy template by ID
pub async fn get_strategy_template(
    template_service: web::Data<StrategyTemplateService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let template_id = path.into_inner();

    match template_service.get_template(&template_id) {
        Some(template) => Ok(HttpResponse::Ok().json(template)),
        None => Err(AppError::NotFound("Template not found".to_string())),
    }
}

/// Run backtest for a strategy template
pub async fn run_template_backtest(
    template_service: web::Data<StrategyTemplateService>,
    req: HttpRequest,
    body: web::Json<BacktestTemplateRequest>,
) -> Result<HttpResponse, AppError> {
    // Get user from JWT claims
    let claims = req.extensions()
        .get::<Claims>()
        .cloned()
        .ok_or_else(|| AppError::Unauthorized("Invalid token".to_string()))?;

    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    info!("Running template backtest for user {} with template {}", user_id, body.template_id);

    // Verify template exists in service
    let template = template_service.get_template(&body.template_id)
        .ok_or_else(|| AppError::NotFound("Template not found".to_string()))?;

    // Map template to strategy name based on template parameters
    let strategy_name = match template.parameters.strategy_type {
        DCAStrategyType::AdaptiveZone => "dca",
        DCAStrategyType::Classic => "dca",
        DCAStrategyType::Aggressive => "dca",
    };

    // Validate inputs
    if body.start_time >= body.end_time {
        return Err(AppError::BadRequest("Start time must be before end time".to_string()));
    }

    if body.initial_balance <= Decimal::ZERO {
        return Err(AppError::BadRequest("Initial balance must be positive".to_string()));
    }

    // Create strategy instance
    let strategy = StrategyRegistry::create_strategy(strategy_name)?;

    // Create backtest config
    let config = BacktestConfig {
        symbol: body.symbol.clone(),
        interval: body.interval.clone(),
        start_time: body.start_time,
        end_time: body.end_time,
        initial_balance: body.initial_balance,
        strategy_name: strategy_name.to_string(),
        strategy_parameters: body.template_parameters.clone(),
    };

    // Run backtest
    let engine = BacktestEngine::new();
    let result = engine.run_backtest(config, strategy).await?;

    let response = TemplateBacktestResponse {
        template_id: body.template_id.clone(),
        template_name: template.name.clone(),
        backtest_result: result,
        user_id: user_id.to_string(),
        created_at: Utc::now(),
    };

    info!("Template backtest completed successfully for user {}", user_id);

    Ok(HttpResponse::Ok().json(response))
}


// Helper functions

// Removed create_template_from_strategy - now using templates from StrategyTemplateService


/// Configure routes for strategy templates
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/strategy-templates")
            .route("", web::get().to(get_strategy_templates))
            .route("/{template_id}", web::get().to(get_strategy_template))
            .route("/{template_id}/backtest", web::post().to(run_template_backtest))
    );
}
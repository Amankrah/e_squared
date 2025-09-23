use actix_web::{web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use dashmap::DashMap;
use tracing::{info, error};

use crate::models::exchange_connection::{
    self, Entity as ExchangeConnection, CreateExchangeConnectionRequest,
    ExchangeConnectionResponse,
};
use crate::exchange_connectors::{
    Exchange, ExchangeFactory, ExchangeCredentials,
    types::{AccountBalances, SpotAccount, MarginAccount, FuturesAccount, FuturesType, WalletType},
    factory::FullExchangeAPI,
};
use crate::utils::{
    errors::AppError,
    encryption::EncryptionService,
};

pub struct ExchangeConnectorManager {
    db: DatabaseConnection,
    encryption_service: Arc<EncryptionService>,
    connectors: Arc<DashMap<Uuid, Arc<dyn FullExchangeAPI>>>,
}

impl ExchangeConnectorManager {
    pub fn new(db: DatabaseConnection, encryption_service: Arc<EncryptionService>) -> Self {
        Self {
            db,
            encryption_service,
            connectors: Arc::new(DashMap::new()),
        }
    }

    async fn get_or_create_connector(
        &self,
        connection_id: Uuid,
    ) -> Result<Arc<dyn FullExchangeAPI>, AppError> {
        if let Some(connector) = self.connectors.get(&connection_id) {
            return Ok(connector.clone());
        }

        let connection = ExchangeConnection::find_by_id(connection_id)
            .one(&self.db)
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

        let exchange = Exchange::from_str(&connection.exchange_name)
            .ok_or_else(|| AppError::BadRequest(format!("Unsupported exchange: {}", connection.exchange_name)))?;

        // For now, return an error indicating that decryption requires user context
        // TODO: Implement proper credential management for background tasks
        return Err(AppError::BadRequest("Exchange connector creation requires user authentication context".to_string()));
    }
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub connection_id: Uuid,
    pub exchange_name: String,
    pub display_name: String,
    pub accounts: AccountBalances,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct GetAccountRequest {
    pub connection_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct GetWalletBalanceRequest {
    pub connection_id: Uuid,
    pub wallet_type: String,
}

#[derive(Debug, Serialize)]
pub struct SpotAccountResponse {
    pub connection_id: Uuid,
    pub exchange_name: String,
    pub account: SpotAccount,
}

#[derive(Debug, Serialize)]
pub struct MarginAccountResponse {
    pub connection_id: Uuid,
    pub exchange_name: String,
    pub account: MarginAccount,
}

#[derive(Debug, Serialize)]
pub struct FuturesAccountResponse {
    pub connection_id: Uuid,
    pub exchange_name: String,
    pub account: FuturesAccount,
}

pub async fn get_all_accounts(
    query: web::Query<serde_json::Value>,
    db: web::Data<DatabaseConnection>,
    _encryption_service: web::Data<Arc<EncryptionService>>,
) -> Result<HttpResponse, AppError> {
    // Get user_id from query parameters instead of JWT
    let user_id_str = query.get("user_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("user_id parameter required".to_string()))?;

    let user_id = Uuid::parse_str(user_id_str)
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let connections = ExchangeConnection::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .filter(exchange_connection::Column::IsActive.eq(true))
        .all(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;

    // For now, return empty responses for each connection
    // TODO: Implement proper exchange connector integration with credential management
    let responses: Vec<AccountResponse> = connections
        .into_iter()
        .map(|connection| AccountResponse {
            connection_id: connection.id,
            exchange_name: connection.exchange_name.clone(),
            display_name: connection.display_name.clone(),
            accounts: AccountBalances {
                spot: None,
                margin: None,
                isolated_margin: None,
                futures_usdm: None,
                futures_coinm: None,
                earn: None,
                total_usd_value: rust_decimal::Decimal::ZERO,
                total_btc_value: rust_decimal::Decimal::ZERO,
            },
            last_update: chrono::Utc::now(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}

pub async fn get_spot_account(
    query: web::Query<GetAccountRequest>,
    db: web::Data<DatabaseConnection>,
    encryption_service: web::Data<Arc<EncryptionService>>,
) -> Result<HttpResponse, AppError> {
    let connection = ExchangeConnection::find_by_id(query.connection_id)
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // For now, return a placeholder response
    // TODO: Implement proper exchange connector integration
    let account = SpotAccount {
        balances: Vec::new(),
        total_usd_value: Some(rust_decimal::Decimal::ZERO),
        total_btc_value: Some(rust_decimal::Decimal::ZERO),
        maker_commission: Some(rust_decimal::Decimal::ZERO),
        taker_commission: Some(rust_decimal::Decimal::ZERO),
        can_trade: false,
        can_withdraw: false,
        can_deposit: false,
        last_update_time: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(SpotAccountResponse {
        connection_id: connection.id,
        exchange_name: connection.exchange_name,
        account,
    }))
}

pub async fn get_margin_account(
    query: web::Query<GetAccountRequest>,
    db: web::Data<DatabaseConnection>,
    encryption_service: web::Data<Arc<EncryptionService>>,
) -> Result<HttpResponse, AppError> {
    let connection = ExchangeConnection::find_by_id(query.connection_id)
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // For now, return a placeholder response
    // TODO: Implement proper exchange connector integration
    let account = MarginAccount {
        balances: Vec::new(),
        total_asset_value: rust_decimal::Decimal::ZERO,
        total_liability_value: rust_decimal::Decimal::ZERO,
        total_net_value: rust_decimal::Decimal::ZERO,
        margin_level: Some(rust_decimal::Decimal::ZERO),
        margin_ratio: Some(rust_decimal::Decimal::ZERO),
        is_margin_enabled: false,
        can_trade: false,
        can_borrow: false,
        last_update_time: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(MarginAccountResponse {
        connection_id: connection.id,
        exchange_name: connection.exchange_name,
        account,
    }))
}

pub async fn get_futures_account(
    query: web::Query<GetAccountRequest>,
    db: web::Data<DatabaseConnection>,
    encryption_service: web::Data<Arc<EncryptionService>>,
) -> Result<HttpResponse, AppError> {
    let connection = ExchangeConnection::find_by_id(query.connection_id)
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // For now, return a placeholder response
    // TODO: Implement proper exchange connector integration
    let account = FuturesAccount {
        account_type: FuturesType::USDM,
        balances: Vec::new(),
        positions: Vec::new(),
        total_wallet_balance: rust_decimal::Decimal::ZERO,
        total_unrealized_pnl: rust_decimal::Decimal::ZERO,
        total_margin_balance: rust_decimal::Decimal::ZERO,
        available_balance: rust_decimal::Decimal::ZERO,
        max_withdraw_amount: rust_decimal::Decimal::ZERO,
        total_initial_margin: rust_decimal::Decimal::ZERO,
        total_maintenance_margin: rust_decimal::Decimal::ZERO,
        margin_ratio: Some(rust_decimal::Decimal::ZERO),
        can_trade: false,
        can_deposit: false,
        can_withdraw: false,
        last_update_time: chrono::Utc::now(),
    };

    Ok(HttpResponse::Ok().json(FuturesAccountResponse {
        connection_id: connection.id,
        exchange_name: connection.exchange_name,
        account,
    }))
}

pub async fn test_exchange_connection(
    query: web::Query<GetAccountRequest>,
    db: web::Data<DatabaseConnection>,
    encryption_service: web::Data<Arc<EncryptionService>>,
) -> Result<HttpResponse, AppError> {
    let connection = ExchangeConnection::find_by_id(query.connection_id)
        .one(db.as_ref())
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or_else(|| AppError::NotFound("Exchange connection not found".to_string()))?;

    // For now, return a placeholder response indicating connection is pending
    // TODO: Implement proper exchange connector integration
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "connection_id": connection.id,
        "exchange_name": connection.exchange_name,
        "is_connected": false,
        "status": "pending_implementation",
        "message": "Exchange connector implementation in progress",
        "tested_at": chrono::Utc::now(),
    })))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/exchange-accounts")
            .route("/all", web::get().to(get_all_accounts))
            .route("/spot", web::get().to(get_spot_account))
            .route("/margin", web::get().to(get_margin_account))
            .route("/futures", web::get().to(get_futures_account))
            .route("/test", web::get().to(test_exchange_connection))
    );
}
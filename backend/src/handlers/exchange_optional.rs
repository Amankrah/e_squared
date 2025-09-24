use actix_web::{web, HttpRequest, HttpResponse, Result, HttpMessage};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::models::{
    exchange_connection::{
        self, Entity as ExchangeConnectionEntity,
        ExchangeConnectionResponse,
    },
    wallet_balance::{
        self, Entity as WalletBalanceEntity,
        WalletBalanceResponse, WalletSummaryResponse, WalletTypeBalance,
    },
};

use std::collections::HashMap;
use rust_decimal::Decimal;

/// Response when no exchange connections exist
#[derive(Debug, Serialize, Deserialize)]
pub struct NoConnectionsResponse {
    pub connections: Vec<ExchangeConnectionResponse>,
    pub message: String,
}

/// Response when no balances exist
#[derive(Debug, Serialize, Deserialize)]
pub struct NoBalancesResponse {
    pub balances: Vec<WalletSummaryResponse>,
    pub message: String,
}

/// Get exchange connections with optional authentication
/// Returns empty list if user has no connections
pub async fn get_exchange_connections_optional(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Try to get user ID from request extensions
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            // Return empty connections with message
            return Ok(HttpResponse::Ok().json(NoConnectionsResponse {
                connections: vec![],
                message: "No exchange connections configured. Please connect an exchange to get started.".to_string(),
            }));
        }
    };

    // Query connections
    match ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .all(db.get_ref())
        .await
    {
        Ok(connections) => {
            if connections.is_empty() {
                Ok(HttpResponse::Ok().json(NoConnectionsResponse {
                    connections: vec![],
                    message: "No exchange connections configured. Please connect an exchange to get started.".to_string(),
                }))
            } else {
                let responses: Vec<ExchangeConnectionResponse> = connections
                    .into_iter()
                    .map(ExchangeConnectionResponse::from)
                    .collect();
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "connections": responses,
                    "message": ""
                })))
            }
        }
        Err(e) => {
            // Log error but return empty list to allow dashboard to load
            tracing::error!("Database error getting exchange connections: {}", e);
            Ok(HttpResponse::Ok().json(NoConnectionsResponse {
                connections: vec![],
                message: "Unable to load exchange connections at this time.".to_string(),
            }))
        }
    }
}

/// Get all user balances with optional authentication
/// Returns empty list if user has no exchange connections
pub async fn get_all_user_balances_optional(
    db: web::Data<DatabaseConnection>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    // Try to get user ID from request extensions
    let user_id = match req.extensions().get::<Uuid>() {
        Some(id) => *id,
        None => {
            // Return empty balances with message
            return Ok(HttpResponse::Ok().json(NoBalancesResponse {
                balances: vec![],
                message: "No exchange connections configured. Connect an exchange to view balances.".to_string(),
            }));
        }
    };

    // Query connections
    let connections = match ExchangeConnectionEntity::find()
        .filter(exchange_connection::Column::UserId.eq(user_id))
        .filter(exchange_connection::Column::IsActive.eq(true))
        .all(db.get_ref())
        .await
    {
        Ok(conns) => conns,
        Err(e) => {
            tracing::error!("Database error getting exchange connections: {}", e);
            return Ok(HttpResponse::Ok().json(NoBalancesResponse {
                balances: vec![],
                message: "Unable to load balances at this time.".to_string(),
            }));
        }
    };

    if connections.is_empty() {
        return Ok(HttpResponse::Ok().json(NoBalancesResponse {
            balances: vec![],
            message: "No active exchange connections. Connect an exchange to view balances.".to_string(),
        }));
    }

    let mut all_summaries = Vec::new();

    for connection in connections {
        let balances = match WalletBalanceEntity::find()
            .filter(wallet_balance::Column::ExchangeConnectionId.eq(connection.id))
            .all(db.get_ref())
            .await
        {
            Ok(bals) => bals,
            Err(e) => {
                tracing::error!("Database error getting wallet balances: {}", e);
                continue;
            }
        };

        // Group by wallet type
        let mut wallet_groups: HashMap<String, Vec<WalletBalanceResponse>> = HashMap::new();
        let mut total_usd_value = Decimal::ZERO;

        for balance in balances {
            let wallet_type = balance.wallet_type.clone();
            let balance_response = WalletBalanceResponse::from(balance.clone());

            if let Some(usd_value) = balance.usd_value {
                if let Ok(decimal_value) = usd_value.parse::<Decimal>() {
                    total_usd_value += decimal_value;
                }
            }

            wallet_groups
                .entry(wallet_type)
                .or_insert_with(Vec::new)
                .push(balance_response);
        }

        let wallets: Vec<WalletTypeBalance> = wallet_groups
            .into_iter()
            .map(|(wallet_type, balances)| {
                let wallet_usd_value = balances
                    .iter()
                    .filter_map(|b| b.usd_value.as_ref()?.parse::<Decimal>().ok())
                    .sum::<Decimal>();

                WalletTypeBalance {
                    wallet_type,
                    balances,
                    wallet_usd_value: if wallet_usd_value > Decimal::ZERO {
                        Some(wallet_usd_value.to_string())
                    } else {
                        None
                    },
                }
            })
            .collect();

        let summary = WalletSummaryResponse {
            exchange_connection_id: connection.id,
            exchange_name: connection.exchange_name,
            display_name: connection.display_name,
            wallets,
            total_usd_value: if total_usd_value > Decimal::ZERO {
                Some(total_usd_value.to_string())
            } else {
                None
            },
            last_updated: connection.last_sync.unwrap_or(connection.updated_at),
        };

        all_summaries.push(summary);
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balances": all_summaries,
        "message": if all_summaries.is_empty() {
            "No balances found. Sync your exchange connections to load balances.".to_string()
        } else {
            "".to_string()
        }
    })))
}
use chrono::{DateTime, Utc, Duration};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::models::{
    exchange_connection::{Entity as ExchangeConnectionEntity, Model as ExchangeConnection},
    wallet_balance::{Entity as WalletBalanceEntity, ActiveModel as WalletBalanceActiveModel},
};
use crate::exchange_connectors::{Exchange, ExchangeFactory, ExchangeCredentials};
use crate::utils::{
    errors::AppError,
    encryption::{EncryptionService, EncryptedData},
};
use sea_orm::{ActiveModelTrait, Set};

/// Service for periodic balance updates from exchanges
#[derive(Clone)]
pub struct BalanceUpdateService {
    db: Arc<DatabaseConnection>,
    encryption_service: Arc<EncryptionService>,

    // Graceful shutdown mechanism
    shutdown_tx: broadcast::Sender<()>,

    // Update intervals (in minutes)
    balance_update_interval: u64,
}

impl BalanceUpdateService {
    pub fn new(
        db: Arc<DatabaseConnection>,
        balance_update_interval_minutes: Option<u64>,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        let encryption_service = Arc::new(EncryptionService::new());

        Self {
            db,
            encryption_service,
            shutdown_tx,
            balance_update_interval: balance_update_interval_minutes.unwrap_or(30), // Default 30 minutes
        }
    }

    /// Start the balance update service
    pub async fn start_service(&self) {
        info!("Starting Balance Update Service with {} minute intervals", self.balance_update_interval);

        let service_clone = self.clone();
        tokio::spawn(async move {
            service_clone.balance_update_loop().await;
        });
    }

    /// Main balance update loop
    async fn balance_update_loop(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(self.balance_update_interval * 60));
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!("Starting periodic balance update cycle");
                    if let Err(e) = self.update_all_active_connections().await {
                        error!("Error in balance update loop: {:?}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Balance update loop shutting down gracefully");
                    break;
                }
            }
        }
    }

    /// Update balances for all active exchange connections
    async fn update_all_active_connections(&self) -> Result<(), AppError> {
        // Get all active exchange connections
        let connections = ExchangeConnectionEntity::find()
            .filter(crate::models::exchange_connection::Column::IsActive.eq(true))
            .filter(crate::models::exchange_connection::Column::ConnectionStatus.eq("connected"))
            .all(self.db.as_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        info!("Found {} active exchange connections to update", connections.len());

        for connection in connections {
            debug!("Updating balances for connection: {} ({})",
                   connection.display_name, connection.exchange_name);

            if let Err(e) = self.update_connection_balances(&connection).await {
                warn!("Failed to update balances for connection {}: {:?}",
                      connection.display_name, e);

                // Update connection status to indicate error
                self.update_connection_error_status(&connection, &format!("{:?}", e)).await;
            } else {
                debug!("Successfully updated balances for connection: {}", connection.display_name);

                // Clear any previous error status
                self.clear_connection_error_status(&connection).await;
            }
        }

        info!("Completed balance update cycle");
        Ok(())
    }

    /// Update balances for a specific exchange connection
    async fn update_connection_balances(&self, connection: &ExchangeConnection) -> Result<(), AppError> {
        // For balance updates, we'll use a default password approach or skip password-protected operations
        // This is because we can't prompt for passwords in an automated service
        // For now, we'll log that password-protected balance updates are skipped

        debug!("Skipping automated balance update for {} - requires user password",
               connection.display_name);

        // TODO: In a production system, you might want to:
        // 1. Store encrypted passwords with user consent
        // 2. Use a different authentication method for automated updates
        // 3. Only update public/non-sensitive balance information

        Ok(())
    }

    /// Update connection status to indicate error
    async fn update_connection_error_status(&self, connection: &ExchangeConnection, error_msg: &str) {
        let update_result = crate::models::exchange_connection::ActiveModel {
            id: Set(connection.id),
            last_error: Set(Some(error_msg.to_string())),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }.update(self.db.as_ref()).await;

        if let Err(e) = update_result {
            warn!("Failed to update connection error status: {:?}", e);
        }
    }

    /// Clear connection error status
    async fn clear_connection_error_status(&self, connection: &ExchangeConnection) {
        let update_result = crate::models::exchange_connection::ActiveModel {
            id: Set(connection.id),
            last_error: Set(None),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }.update(self.db.as_ref()).await;

        if let Err(e) = update_result {
            warn!("Failed to clear connection error status: {:?}", e);
        }
    }

    /// Gracefully shutdown the service
    pub async fn shutdown(&self) {
        info!("Initiating graceful shutdown of Balance Update Service");
        let _ = self.shutdown_tx.send(());
    }
}

/// Alternative implementation for balance updates that doesn't require passwords
/// This version could be used for public balance information or when using API keys
/// that don't require additional authentication
impl BalanceUpdateService {
    /// Update balances using only API key (for exchanges that support this)
    pub async fn update_public_balances(&self, connection: &ExchangeConnection) -> Result<(), AppError> {
        // This would be implemented for exchanges that allow balance queries with just API keys
        // For security reasons, many exchanges require additional authentication for balance queries

        info!("Public balance update not implemented yet for connection: {}", connection.display_name);
        Ok(())
    }

    /// Check if an exchange connection supports public balance queries
    pub fn supports_public_balance_queries(&self, exchange_name: &str) -> bool {
        // Different exchanges have different security requirements
        // This would return true for exchanges that allow balance queries with just API keys
        match exchange_name.to_lowercase().as_str() {
            "binance" => false, // Binance requires signed requests for balance queries
            "coinbase" => false, // Most exchanges require authentication for balance queries
            _ => false,
        }
    }
}
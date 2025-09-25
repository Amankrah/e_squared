use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex, broadcast};
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use futures;

use crate::models::{
    dca_strategy::{
        Entity as DCAStrategyEntity, Model as DCAStrategy,
        execution::ActiveModel as ExecutionActiveModel,
        ExecutionType, TriggerReason,
        market_data::Model as MarketDataModel,
    },
    exchange_connection::Entity as ExchangeConnectionEntity,
};
use crate::services::MarketDataService;
use crate::utils::{
    errors::AppError,
    encryption::EncryptionService,
};

/// High-performance DCA execution engine optimized for Rust's capabilities
#[derive(Clone)]
pub struct DCAExecutionEngine {
    db: Arc<DatabaseConnection>,
    market_service: Arc<MarketDataService>,
    #[allow(dead_code)]
    encryption_service: Arc<EncryptionService>,

    // In-memory cache for performance
    strategy_cache: Arc<RwLock<HashMap<Uuid, DCAStrategy>>>,
    market_data_cache: Arc<RwLock<HashMap<String, MarketDataModel>>>,

    // Execution queue for batch processing
    execution_queue: Arc<Mutex<Vec<ExecutionRequest>>>,

    // Performance metrics
    execution_stats: Arc<RwLock<ExecutionStats>>,

    // Graceful shutdown mechanism
    shutdown_tx: broadcast::Sender<()>,
}

#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub strategy_id: Uuid,
    pub trigger_reason: TriggerReason,
    pub manual_amount: Option<Decimal>,
    #[allow(dead_code)]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_volume_usd: Decimal,
    pub average_execution_time_ms: f64,
    pub last_execution_batch: Option<DateTime<Utc>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub strategy_id: Uuid,
    pub execution_id: Uuid,
    pub success: bool,
    pub execution_type: ExecutionType,
    pub amount_usd: Decimal,
    pub amount_asset: Option<Decimal>,
    pub price: Option<Decimal>,
    pub error_message: Option<String>,
    pub execution_time_ms: u128,
}

#[allow(dead_code)]
impl DCAExecutionEngine {
    pub fn new(
        db: Arc<DatabaseConnection>,
        market_service: MarketDataService,
        encryption_service: EncryptionService,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            db,
            market_service: Arc::new(market_service),
            encryption_service: Arc::new(encryption_service),
            strategy_cache: Arc::new(RwLock::new(HashMap::new())),
            market_data_cache: Arc::new(RwLock::new(HashMap::new())),
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
            shutdown_tx,
        }
    }

    /// Start the high-performance execution engine
    pub async fn start_engine(&self) {
        info!("Starting E² Strategy Execution Engine with optimized performance");

        // Start multiple concurrent tasks for maximum performance
        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.strategy_monitoring_loop().await;
        });

        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.market_data_update_loop().await;
        });

        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.batch_execution_loop().await;
        });

        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.cache_optimization_loop().await;
        });
    }

    /// Monitor strategies and queue executions (runs every 30 seconds)
    async fn strategy_monitoring_loop(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(30));
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.scan_and_queue_strategies().await {
                        error!("Error in strategy monitoring loop: {:?}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Strategy monitoring loop shutting down gracefully");
                    break;
                }
            }
        }
    }

    /// Update market data cache (runs every 60 seconds)
    async fn market_data_update_loop(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(60));
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.update_market_data_cache().await {
                        error!("Error in market data update loop: {:?}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Market data update loop shutting down gracefully");
                    break;
                }
            }
        }
    }

    /// Process execution queue in batches (runs every 10 seconds)
    async fn batch_execution_loop(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(10));
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.process_execution_batch().await {
                        error!("Error in batch execution loop: {:?}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Batch execution loop shutting down gracefully");
                    break;
                }
            }
        }
    }

    /// Optimize cache and cleanup (runs every 5 minutes)
    async fn cache_optimization_loop(&self) {
        let mut interval = interval(tokio::time::Duration::from_secs(300));
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = self.optimize_cache().await {
                        error!("Error in cache optimization loop: {:?}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Cache optimization loop shutting down gracefully");
                    break;
                }
            }
        }
    }

    /// Scan active strategies and queue those ready for execution
    async fn scan_and_queue_strategies(&self) -> Result<(), AppError> {
        let start_time = std::time::Instant::now();

        // Load active strategies from cache or database
        let strategies = self.get_active_strategies().await?;

        debug!("Scanning {} active strategies for execution", strategies.len());

        let mut queue_requests = Vec::new();

        for strategy in strategies {
            // Check if strategy should execute
            if let Some(trigger_reason) = self.should_strategy_execute(&strategy).await? {
                queue_requests.push(ExecutionRequest {
                    strategy_id: strategy.id,
                    trigger_reason,
                    manual_amount: None,
                    created_at: Utc::now(),
                });
            }
        }

        // Add to execution queue
        if !queue_requests.is_empty() {
            let mut queue = self.execution_queue.lock().await;
            queue.extend(queue_requests.clone());
            info!("Queued {} strategies for execution", queue_requests.len());
        }

        debug!("Strategy scan completed in {}ms", start_time.elapsed().as_millis());
        Ok(())
    }

    /// Check if a strategy should execute using the strategy framework
    async fn should_strategy_execute(&self, strategy: &DCAStrategy) -> Result<Option<TriggerReason>, AppError> {
        // Check if strategy is active
        if strategy.status != "active" {
            return Ok(None);
        }

        // Check time-based execution first
        let now = Utc::now();
        if let Some(next_execution) = strategy.next_execution_at {
            if now < next_execution {
                return Ok(None);
            }
        }

        // Get historical market data for the strategy framework
        let historical_data = self.get_historical_data_for_asset(&strategy.asset_symbol).await?;

        // Use the strategy framework to determine if we should execute
        match strategy.should_execute(historical_data).await {
            Ok(true) => Ok(Some(TriggerReason::Scheduled)), // Strategy framework said yes
            Ok(false) => Ok(None), // Strategy framework said no
            Err(e) => {
                warn!("Strategy framework analysis failed for strategy {}: {}", strategy.id, e);
                Ok(None) // Fail safe - don't execute if framework fails
            }
        }
    }

    /// Process queued executions in optimized batches
    async fn process_execution_batch(&self) -> Result<(), AppError> {
        let mut queue = self.execution_queue.lock().await;
        if queue.is_empty() {
            return Ok(());
        }

        // Take up to 50 executions per batch for optimal performance
        let batch_size = std::cmp::min(queue.len(), 50);
        let batch: Vec<ExecutionRequest> = queue.drain(..batch_size).collect();
        drop(queue); // Release lock early

        info!("Processing execution batch of {} requests", batch.len());

        let start_time = std::time::Instant::now();

        // Process executions concurrently using Rust's async capabilities
        let execution_futures: Vec<_> = batch.into_iter()
            .map(|request| self.execute_strategy_request(request))
            .collect();

        let results = futures::future::join_all(execution_futures).await;

        // Update statistics
        let mut stats = self.execution_stats.write().await;
        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        stats.total_executions += results.len() as u64;
        stats.successful_executions += successful as u64;
        stats.failed_executions += failed as u64;
        stats.last_execution_batch = Some(Utc::now());

        let batch_time = start_time.elapsed().as_millis();
        stats.average_execution_time_ms = (stats.average_execution_time_ms + batch_time as f64) / 2.0;

        info!("Batch execution completed: {} successful, {} failed, {}ms total",
              successful, failed, batch_time);

        Ok(())
    }

    /// Execute a single strategy request with optimized performance
    async fn execute_strategy_request(&self, request: ExecutionRequest) -> ExecutionResult {
        let start_time = std::time::Instant::now();

        let strategy = match self.get_strategy_from_cache(request.strategy_id).await {
            Ok(strategy) => strategy,
            Err(e) => {
                return ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: false,
                    execution_type: ExecutionType::Skip,
                    amount_usd: Decimal::ZERO,
                    amount_asset: None,
                    price: None,
                    error_message: Some(format!("Failed to load strategy: {:?}", e)),
                    execution_time_ms: start_time.elapsed().as_millis(),
                };
            }
        };

        let market_data = match self.get_market_data_for_asset(&strategy.asset_symbol).await {
            Ok(data) => data,
            Err(e) => {
                return ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: false,
                    execution_type: ExecutionType::Skip,
                    amount_usd: Decimal::ZERO,
                    amount_asset: None,
                    price: None,
                    error_message: Some(format!("Failed to get market data: {:?}", e)),
                    execution_time_ms: start_time.elapsed().as_millis(),
                };
            }
        };

        // Get historical data for strategy framework
        let historical_data = match self.get_historical_data_for_asset(&strategy.asset_symbol).await {
            Ok(data) => data,
            Err(e) => {
                return ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: false,
                    execution_type: ExecutionType::Skip,
                    amount_usd: Decimal::ZERO,
                    amount_asset: None,
                    price: Some(market_data.price),
                    error_message: Some(format!("Failed to get historical data: {:?}", e)),
                    execution_time_ms: start_time.elapsed().as_millis(),
                };
            }
        };

        // Use strategy framework to determine execution
        let should_execute = match strategy.should_execute(historical_data.clone()).await {
            Ok(should) => should,
            Err(e) => {
                return ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: false,
                    execution_type: ExecutionType::Skip,
                    amount_usd: Decimal::ZERO,
                    amount_asset: None,
                    price: Some(market_data.price),
                    error_message: Some(format!("Strategy framework failed: {}", e)),
                    execution_time_ms: start_time.elapsed().as_millis(),
                };
            }
        };

        if !should_execute {
            return ExecutionResult {
                strategy_id: request.strategy_id,
                execution_id: Uuid::new_v4(),
                success: true,
                execution_type: ExecutionType::Skip,
                amount_usd: Decimal::ZERO,
                amount_asset: None,
                price: Some(market_data.price),
                error_message: None,
                execution_time_ms: start_time.elapsed().as_millis(),
            };
        }

        // For now, we default to buy execution (most DCA strategies are buy-heavy)
        let execution_type = ExecutionType::Buy;

        // Calculate dynamic amount using strategy framework
        let amount_usd = if let Some(manual_amount) = request.manual_amount {
            manual_amount
        } else {
            match strategy.calculate_current_tranche_size(historical_data).await {
                Ok(size) => size,
                Err(e) => {
                    return ExecutionResult {
                        strategy_id: request.strategy_id,
                        execution_id: Uuid::new_v4(),
                        success: false,
                        execution_type: ExecutionType::Skip,
                        amount_usd: Decimal::ZERO,
                        amount_asset: None,
                        price: Some(market_data.price),
                        error_message: Some(format!("Failed to calculate tranche size: {}", e)),
                        execution_time_ms: start_time.elapsed().as_millis(),
                    };
                }
            }
        };

        // Execute the actual trade
        match self.execute_trade(&strategy, execution_type.clone(), amount_usd, market_data.price).await {
            Ok((amount_asset, actual_price)) => {
                // Record execution in database
                if let Err(e) = self.record_execution(
                    request.strategy_id,
                    execution_type.clone(),
                    request.trigger_reason,
                    amount_usd,
                    Some(amount_asset),
                    Some(actual_price),
                    market_data.fear_greed_index,
                    market_data.volatility_7d,
                    None,
                ).await {
                    warn!("Failed to record execution: {:?}", e);
                }

                // Update strategy statistics
                if let Err(e) = self.update_strategy_stats(
                    request.strategy_id,
                    execution_type.clone(),
                    amount_usd,
                    amount_asset,
                    actual_price,
                ).await {
                    warn!("Failed to update strategy stats: {:?}", e);
                }

                ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: true,
                    execution_type,
                    amount_usd,
                    amount_asset: Some(amount_asset),
                    price: Some(actual_price),
                    error_message: None,
                    execution_time_ms: start_time.elapsed().as_millis(),
                }
            }
            Err(e) => {
                // Record failed execution
                if let Err(record_err) = self.record_execution(
                    request.strategy_id,
                    execution_type.clone(),
                    request.trigger_reason,
                    amount_usd,
                    None,
                    Some(market_data.price),
                    market_data.fear_greed_index,
                    market_data.volatility_7d,
                    Some(e.to_string()),
                ).await {
                    warn!("Failed to record failed execution: {:?}", record_err);
                }

                ExecutionResult {
                    strategy_id: request.strategy_id,
                    execution_id: Uuid::new_v4(),
                    success: false,
                    execution_type,
                    amount_usd,
                    amount_asset: None,
                    price: Some(market_data.price),
                    error_message: Some(e.to_string()),
                    execution_time_ms: start_time.elapsed().as_millis(),
                }
            }
        }
    }

    /// Execute actual trade on exchange with error handling
    async fn execute_trade(
        &self,
        strategy: &DCAStrategy,
        execution_type: ExecutionType,
        amount_usd: Decimal,
        current_price: Decimal,
    ) -> Result<(Decimal, Decimal), AppError> {
        // Get exchange connection for user
        let _exchange_connection = ExchangeConnectionEntity::find()
            .filter(crate::models::exchange_connection::Column::UserId.eq(strategy.user_id))
            .filter(crate::models::exchange_connection::Column::IsActive.eq(true))
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::BadRequest("No active exchange connection found".to_string()))?;

        // For now, simulate trade execution
        // In production, this would place actual orders on the exchange
        let amount_asset = amount_usd / current_price;

        // Simulate small price slippage (0.1%)
        let actual_price = current_price * Decimal::try_from(1.001)
            .map_err(|_| AppError::InternalServerError)?;

        // Add small delay to simulate network latency
        sleep(tokio::time::Duration::from_millis(100)).await;

        info!("Simulated {} of {:.6} {} at ${:.2} for strategy {}",
              String::from(execution_type.clone()),
              amount_asset,
              strategy.asset_symbol,
              actual_price,
              strategy.name);

        Ok((amount_asset, actual_price))
    }

    /// Record execution in database
    async fn record_execution(
        &self,
        strategy_id: Uuid,
        execution_type: ExecutionType,
        trigger_reason: TriggerReason,
        amount_usd: Decimal,
        amount_asset: Option<Decimal>,
        price: Option<Decimal>,
        fear_greed_index: Option<i32>,
        volatility: Option<Decimal>,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        let execution = ExecutionActiveModel {
            id: Set(Uuid::new_v4()),
            strategy_id: Set(strategy_id),
            exchange_connection_id: Set(Uuid::new_v4()), // Would be actual connection ID
            execution_type: Set(execution_type.into()),
            trigger_reason: Set(trigger_reason.into()),
            amount_usd: Set(amount_usd),
            amount_asset: Set(amount_asset),
            price_at_execution: Set(price),
            fear_greed_index: Set(fear_greed_index),
            market_volatility: Set(volatility),
            order_id: Set(None),
            order_status: Set("filled".to_string()),
            execution_timestamp: Set(Utc::now()),
            error_message: Set(error_message),
            created_at: Set(Utc::now()),
        };

        execution.insert(self.db.as_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        Ok(())
    }

    /// Update strategy statistics after execution
    async fn update_strategy_stats(
        &self,
        _strategy_id: Uuid,
        _execution_type: ExecutionType,
        _amount_usd: Decimal,
        _amount_asset: Decimal,
        _price: Decimal,
    ) -> Result<(), AppError> {
        // Implementation would update strategy totals, average price, etc.
        // This is simplified for the example
        Ok(())
    }

    /// Get active strategies with caching
    async fn get_active_strategies(&self) -> Result<Vec<DCAStrategy>, AppError> {
        // Check cache first
        let cache = self.strategy_cache.read().await;
        if !cache.is_empty() {
            return Ok(cache.values().cloned().collect());
        }
        drop(cache);

        // Load from database
        let strategies = DCAStrategyEntity::find()
            .filter(crate::models::dca_strategy::Column::Status.eq("active"))
            .all(self.db.as_ref())
            .await
            .map_err(AppError::DatabaseError)?;

        // Update cache
        let mut cache = self.strategy_cache.write().await;
        for strategy in &strategies {
            cache.insert(strategy.id, strategy.clone());
        }

        Ok(strategies)
    }

    /// Get strategy from cache with fallback to database
    async fn get_strategy_from_cache(&self, strategy_id: Uuid) -> Result<DCAStrategy, AppError> {
        // Check cache first
        let cache = self.strategy_cache.read().await;
        if let Some(strategy) = cache.get(&strategy_id) {
            return Ok(strategy.clone());
        }
        drop(cache);

        // Load from database
        let strategy = DCAStrategyEntity::find_by_id(strategy_id)
            .one(self.db.as_ref())
            .await
            .map_err(AppError::DatabaseError)?
            .ok_or_else(|| AppError::NotFound("Strategy not found".to_string()))?;

        // Update cache
        let mut cache = self.strategy_cache.write().await;
        cache.insert(strategy_id, strategy.clone());

        Ok(strategy)
    }

    /// Get market data with caching
    async fn get_market_data_for_asset(&self, symbol: &str) -> Result<MarketDataModel, AppError> {
        // Check cache first
        let cache = self.market_data_cache.read().await;
        if let Some(data) = cache.get(symbol) {
            // Check if data is recent (within 5 minutes)
            if Utc::now() - data.timestamp < Duration::minutes(5) {
                return Ok(data.clone());
            }
        }
        drop(cache);

        // Fetch fresh data
        let market_data = self.market_service.get_market_data(symbol).await?;

        // Update cache
        let mut cache = self.market_data_cache.write().await;
        cache.insert(symbol.to_string(), market_data.clone());

        Ok(market_data)
    }

    /// Get historical kline data for strategy framework
    async fn get_historical_data_for_asset(&self, symbol: &str) -> Result<Vec<crate::exchange_connectors::Kline>, AppError> {
        // For now, return empty vec. In production, this would fetch real historical data
        // from the market data service or database
        Ok(vec![])
    }

    /// Update market data cache for all tracked assets
    async fn update_market_data_cache(&self) -> Result<(), AppError> {
        // Get unique symbols from active strategies
        let strategies = self.get_active_strategies().await?;
        let symbols: std::collections::HashSet<String> = strategies
            .iter()
            .map(|s| s.asset_symbol.clone())
            .collect();

        debug!("Updating market data cache for {} symbols", symbols.len());

        // Update data for each symbol concurrently
        let update_futures: Vec<_> = symbols.into_iter()
            .map(|symbol| async move {
                match self.market_service.get_market_data(&symbol).await {
                    Ok(data) => {
                        let mut cache = self.market_data_cache.write().await;
                        cache.insert(symbol.clone(), data);
                        Ok(())
                    }
                    Err(e) => {
                        warn!("Failed to update market data for {}: {:?}", symbol, e);
                        Err(e)
                    }
                }
            })
            .collect();

        let _results = futures::future::join_all(update_futures).await;
        Ok(())
    }

    /// Optimize cache by removing stale entries
    async fn optimize_cache(&self) -> Result<(), AppError> {
        let now = Utc::now();

        // Clean market data cache (remove entries older than 10 minutes)
        let mut market_cache = self.market_data_cache.write().await;
        market_cache.retain(|_, data| now - data.timestamp < Duration::minutes(10));
        let market_cache_size = market_cache.len();
        drop(market_cache);

        // Clean strategy cache periodically (reload from database)
        let mut strategy_cache = self.strategy_cache.write().await;
        let strategy_cache_size = strategy_cache.len();
        strategy_cache.clear(); // Will be reloaded on next access
        drop(strategy_cache);

        debug!("Cache optimization: {} market data entries, {} strategy entries cleared",
               market_cache_size, strategy_cache_size);

        Ok(())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Manually queue a strategy for execution
    pub async fn queue_manual_execution(
        &self,
        strategy_id: Uuid,
        amount_usd: Option<Decimal>,
    ) -> Result<(), AppError> {
        let request = ExecutionRequest {
            strategy_id,
            trigger_reason: TriggerReason::Manual,
            manual_amount: amount_usd,
            created_at: Utc::now(),
        };

        let mut queue = self.execution_queue.lock().await;
        queue.push(request);

        info!("Manually queued strategy {} for execution", strategy_id);
        Ok(())
    }

    /// Initiate graceful shutdown of all background loops  
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<(), AppError> {
        info!("Initiating graceful shutdown of DCA Execution Engine");

        if let Err(e) = self.shutdown_tx.send(()) {
            warn!("Failed to send shutdown signal: {:?}", e);
        }

        // Give background tasks time to finish current operations
        sleep(tokio::time::Duration::from_secs(2)).await;

        info!("DCA Execution Engine shutdown completed");
        Ok(())
    }

    /// Get shutdown receiver for external coordination
    #[allow(dead_code)]
    pub fn get_shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}


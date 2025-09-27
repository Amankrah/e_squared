use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::sync::{RwLock, mpsc, Mutex};
use tokio::time::interval;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::strategies::core::{
    Strategy, StrategyContext, StrategySignal, StrategyMode, LiveExecutableStrategy,
    list_all_strategies, create_strategy, StrategyContextBuilder,
};
use crate::utils::errors::AppError;
use super::types::*;
use super::executor::SignalExecutor;
use super::monitor::StrategyMonitor;
use super::scheduler::ExecutionScheduler;
use super::risk_manager::RiskManager;

/// Live execution engine for trading strategies
pub struct ExecutionEngine {
    /// Engine configuration
    config: ExecutionConfig,
    /// Active strategy instances
    instances: Arc<RwLock<HashMap<Uuid, Arc<Mutex<Box<dyn Strategy>>>>>>,
    /// Instance metadata
    instance_metadata: Arc<RwLock<HashMap<Uuid, StrategyInstance>>>,
    /// Signal executor
    signal_executor: Arc<SignalExecutor>,
    /// Strategy monitor
    monitor: Arc<StrategyMonitor>,
    /// Execution scheduler
    scheduler: Arc<ExecutionScheduler>,
    /// Risk manager
    risk_manager: Arc<RiskManager>,
    /// Event channel
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<ExecutionEvent>>>,
    /// Engine state
    is_running: Arc<RwLock<bool>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
}

impl ExecutionEngine {
    /// Create a new execution engine
    pub async fn new(config: ExecutionConfig) -> Result<Self, AppError> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let signal_executor = Arc::new(SignalExecutor::new(config.clone()).await?);
        let monitor = Arc::new(StrategyMonitor::new());
        let scheduler = Arc::new(ExecutionScheduler::new());
        let risk_manager = Arc::new(RiskManager::new(config.risk_config.clone()));

        Ok(Self {
            config,
            instances: Arc::new(RwLock::new(HashMap::new())),
            instance_metadata: Arc::new(RwLock::new(HashMap::new())),
            signal_executor,
            monitor,
            scheduler,
            risk_manager,
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
        })
    }

    /// Start the execution engine
    pub async fn start(&self) -> Result<(), AppError> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(AppError::BadRequest("Engine is already running".to_string()));
            }
            *running = true;
        }

        info!("Starting execution engine");

        // Start background tasks
        self.start_background_tasks().await;

        info!("Execution engine started successfully");
        Ok(())
    }

    /// Stop the execution engine
    pub async fn stop(&self) -> Result<(), AppError> {
        {
            let mut running = self.is_running.write().await;
            if !*running {
                return Ok(()); // Already stopped
            }
            *running = false;
        }

        info!("Stopping execution engine");

        // Stop all strategy instances
        let instance_ids: Vec<Uuid> = {
            let metadata = self.instance_metadata.read().await;
            metadata.keys().cloned().collect()
        };

        for instance_id in instance_ids {
            if let Err(e) = self.stop_strategy_instance(instance_id).await {
                error!("Failed to stop strategy instance {}: {}", instance_id, e);
            }
        }

        info!("Execution engine stopped");
        Ok(())
    }

    /// Create and start a new strategy instance
    pub async fn create_strategy_instance(
        &self,
        user_id: Uuid,
        strategy_id: String,
        symbol: String,
        interval: String,
        config: serde_json::Value,
        mode: StrategyMode,
    ) -> Result<Uuid, AppError> {
        // Check if engine is running
        if !*self.is_running.read().await {
            return Err(AppError::BadRequest("Engine is not running".to_string()));
        }

        // Check concurrent strategy limit
        {
            let instances = self.instances.read().await;
            if instances.len() >= self.config.max_concurrent_strategies {
                return Err(AppError::BadRequest("Maximum concurrent strategies limit reached".to_string()));
            }
        }

        // Create strategy instance
        let mut strategy = create_strategy(&strategy_id)?;
        let instance_id = Uuid::new_v4();

        // Create context for initialization
        let context = StrategyContextBuilder::new()
            .strategy_id(instance_id)
            .user_id(user_id)
            .symbol(symbol.clone())
            .interval(interval.clone())
            .mode(mode.clone())
            .build()?;

        // Initialize strategy
        strategy.initialize(&config, mode.clone(), &context).await?;

        // Create instance metadata
        let instance = StrategyInstance {
            id: instance_id,
            user_id,
            strategy_id: strategy_id.clone(),
            symbol: symbol.clone(),
            interval: interval.clone(),
            mode,
            config,
            status: InstanceStatus::Starting,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_execution: None,
            next_execution: None,
            metrics: InstanceMetrics::default(),
        };

        // Store instance
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, Arc::new(Mutex::new(strategy)));
        }

        {
            let mut metadata = self.instance_metadata.write().await;
            metadata.insert(instance_id, instance);
        }

        // Send event
        let _ = self.event_sender.send(ExecutionEvent::InstanceCreated {
            instance_id,
            user_id,
            strategy_id,
        });

        // Start the strategy if it's a live strategy
        if mode == StrategyMode::Live {
            self.start_strategy_instance(instance_id).await?;
        }

        info!("Created strategy instance {} for user {}", instance_id, user_id);
        Ok(instance_id)
    }

    /// Start a strategy instance
    pub async fn start_strategy_instance(&self, instance_id: Uuid) -> Result<(), AppError> {
        // Update status
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.status = InstanceStatus::Running;
                instance.updated_at = Utc::now();
            }
        }

        // If it's a live strategy, start live execution
        if let Some(strategy_arc) = {
            let instances = self.instances.read().await;
            instances.get(&instance_id).cloned()
        } {
            let mut strategy = strategy_arc.lock().await;

            // Check if strategy supports live execution
            if let Some(live_strategy) = strategy.as_any().downcast_mut::<dyn LiveExecutableStrategy>() {
                // Create context
                let context = self.create_context_for_instance(instance_id).await?;
                live_strategy.start_live_execution(&context).await?;
            }
        }

        // Send event
        let _ = self.event_sender.send(ExecutionEvent::InstanceStarted { instance_id });

        // Schedule the strategy
        self.scheduler.schedule_strategy(instance_id).await;

        info!("Started strategy instance {}", instance_id);
        Ok(())
    }

    /// Stop a strategy instance
    pub async fn stop_strategy_instance(&self, instance_id: Uuid) -> Result<(), AppError> {
        // Update status
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.status = InstanceStatus::Stopping;
                instance.updated_at = Utc::now();
            }
        }

        // Stop live execution if applicable
        if let Some(strategy_arc) = {
            let instances = self.instances.read().await;
            instances.get(&instance_id).cloned()
        } {
            let mut strategy = strategy_arc.lock().await;

            if let Some(live_strategy) = strategy.as_any().downcast_mut::<dyn LiveExecutableStrategy>() {
                live_strategy.stop_live_execution().await?;
            }

            // Call on_stop
            strategy.on_stop().await?;
        }

        // Unschedule the strategy
        self.scheduler.unschedule_strategy(instance_id).await;

        // Update final status
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.status = InstanceStatus::Stopped;
                instance.updated_at = Utc::now();
            }
        }

        // Send event
        let _ = self.event_sender.send(ExecutionEvent::InstanceStopped {
            instance_id,
            reason: "Manually stopped".to_string(),
        });

        info!("Stopped strategy instance {}", instance_id);
        Ok(())
    }

    /// Pause a strategy instance
    pub async fn pause_strategy_instance(&self, instance_id: Uuid) -> Result<(), AppError> {
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.status = InstanceStatus::Paused;
                instance.updated_at = Utc::now();
            }
        }

        // Send event
        let _ = self.event_sender.send(ExecutionEvent::InstancePaused {
            instance_id,
            reason: "Manually paused".to_string(),
        });

        info!("Paused strategy instance {}", instance_id);
        Ok(())
    }

    /// Resume a strategy instance
    pub async fn resume_strategy_instance(&self, instance_id: Uuid) -> Result<(), AppError> {
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.status = InstanceStatus::Running;
                instance.updated_at = Utc::now();
            }
        }

        // Send event
        let _ = self.event_sender.send(ExecutionEvent::InstanceResumed { instance_id });

        info!("Resumed strategy instance {}", instance_id);
        Ok(())
    }

    /// Remove a strategy instance
    pub async fn remove_strategy_instance(&self, instance_id: Uuid) -> Result<(), AppError> {
        // Stop first if running
        if let Ok(instance) = self.get_strategy_instance(instance_id).await {
            if matches!(instance.status, InstanceStatus::Running | InstanceStatus::Paused) {
                self.stop_strategy_instance(instance_id).await?;
            }
        }

        // Remove from collections
        {
            let mut instances = self.instances.write().await;
            instances.remove(&instance_id);
        }

        {
            let mut metadata = self.instance_metadata.write().await;
            metadata.remove(&instance_id);
        }

        info!("Removed strategy instance {}", instance_id);
        Ok(())
    }

    /// Get strategy instance information
    pub async fn get_strategy_instance(&self, instance_id: Uuid) -> Result<StrategyInstance, AppError> {
        let metadata = self.instance_metadata.read().await;
        metadata
            .get(&instance_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Strategy instance not found: {}", instance_id)))
    }

    /// List all strategy instances for a user
    pub async fn list_user_instances(&self, user_id: Uuid) -> Vec<StrategyInstance> {
        let metadata = self.instance_metadata.read().await;
        metadata
            .values()
            .filter(|instance| instance.user_id == user_id)
            .cloned()
            .collect()
    }

    /// List all strategy instances
    pub async fn list_all_instances(&self) -> Vec<StrategyInstance> {
        let metadata = self.instance_metadata.read().await;
        metadata.values().cloned().collect()
    }

    /// Execute a strategy once (for testing or manual execution)
    pub async fn execute_strategy_once(&self, instance_id: Uuid) -> Result<Option<ExecutionResult>, AppError> {
        let context = self.create_context_for_instance(instance_id).await?;

        if let Some(strategy_arc) = {
            let instances = self.instances.read().await;
            instances.get(&instance_id).cloned()
        } {
            let mut strategy = strategy_arc.lock().await;

            // Analyze and get signal
            match strategy.analyze(&context).await {
                Ok(Some(signal)) => {
                    // Execute the signal
                    let result = self.execute_signal(instance_id, signal, &context).await?;
                    Ok(Some(result))
                }
                Ok(None) => {
                    debug!("No signal generated for instance {}", instance_id);
                    Ok(None)
                }
                Err(e) => {
                    error!("Strategy analysis failed for instance {}: {}", instance_id, e);
                    Err(e)
                }
            }
        } else {
            Err(AppError::NotFound("Strategy instance not found".to_string()))
        }
    }

    /// Execute a signal
    async fn execute_signal(
        &self,
        instance_id: Uuid,
        signal: StrategySignal,
        context: &StrategyContext,
    ) -> Result<ExecutionResult, AppError> {
        // Send signal generated event
        let _ = self.event_sender.send(ExecutionEvent::SignalGenerated {
            instance_id,
            signal: signal.clone(),
        });

        // Risk assessment
        let risk_assessment = self.risk_manager.assess_signal(&signal, context).await;
        if !risk_assessment.allowed {
            let result = ExecutionResult {
                signal,
                status: ExecutionStatus::Skipped,
                order_id: None,
                execution_price: None,
                executed_quantity: None,
                fees: None,
                timestamp: Utc::now(),
                error: Some(format!("Risk management blocked: {}", risk_assessment.blocking_reasons.join(", "))),
                execution_time_ms: 0,
            };

            let _ = self.event_sender.send(ExecutionEvent::ExecutionCompleted {
                instance_id,
                result: result.clone(),
            });

            return Ok(result);
        }

        // Execute the signal
        let _ = self.event_sender.send(ExecutionEvent::ExecutionStarted {
            instance_id,
            signal: signal.clone(),
        });

        let result = self.signal_executor.execute_signal(signal, context).await?;

        // Update instance metrics
        {
            let mut metadata = self.instance_metadata.write().await;
            if let Some(instance) = metadata.get_mut(&instance_id) {
                instance.metrics.signals_executed += 1;
                instance.last_execution = Some(Utc::now());

                if result.status == ExecutionStatus::Success {
                    // Update success metrics
                } else {
                    instance.metrics.execution_errors += 1;
                    instance.metrics.last_error = result.error.clone();
                }
            }
        }

        // Send execution completed event
        let _ = self.event_sender.send(ExecutionEvent::ExecutionCompleted {
            instance_id,
            result: result.clone(),
        });

        Ok(result)
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.stats.read().await.clone()
    }

    /// Get available strategies
    pub async fn get_available_strategies(&self) -> Result<Vec<crate::strategies::core::StrategyListItem>, AppError> {
        list_all_strategies()
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        // Task 1: Strategy execution loop
        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.strategy_execution_loop().await;
        });

        // Task 2: Monitoring and maintenance
        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.monitoring_loop().await;
        });

        // Task 3: Event processing
        let engine_clone = self.clone();
        tokio::spawn(async move {
            engine_clone.event_processing_loop().await;
        });
    }

    /// Main strategy execution loop
    async fn strategy_execution_loop(&self) {
        let mut interval = interval(Duration::from_secs(1)); // Check every second

        while *self.is_running.read().await {
            interval.tick().await;

            // Get strategies that need execution
            let strategies_to_execute = self.scheduler.get_due_strategies().await;

            for instance_id in strategies_to_execute {
                if let Ok(instance) = self.get_strategy_instance(instance_id).await {
                    if instance.status == InstanceStatus::Running {
                        if let Err(e) = self.execute_strategy_once(instance_id).await {
                            error!("Failed to execute strategy {}: {}", instance_id, e);
                        }
                    }
                }
            }
        }
    }

    /// Monitoring and maintenance loop
    async fn monitoring_loop(&self) {
        let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds

        while *self.is_running.read().await {
            interval.tick().await;

            // Update statistics
            self.update_statistics().await;

            // Health check on strategies
            self.monitor.health_check(&self.instance_metadata).await;
        }
    }

    /// Event processing loop
    async fn event_processing_loop(&self) {
        let mut receiver = self.event_receiver.lock().await;

        while *self.is_running.read().await {
            if let Some(event) = receiver.recv().await {
                self.process_event(event).await;
            }
        }
    }

    /// Process execution event
    async fn process_event(&self, event: ExecutionEvent) {
        match event {
            ExecutionEvent::ErrorOccurred { instance_id, error, .. } => {
                error!("Strategy instance {} error: {}", instance_id, error);
                // Could implement automatic recovery logic here
            }
            ExecutionEvent::RiskLimitExceeded { instance_id, .. } => {
                warn!("Risk limit exceeded for instance {}, pausing", instance_id);
                let _ = self.pause_strategy_instance(instance_id).await;
            }
            ExecutionEvent::OrderFilled { instance_id, order_id, symbol, filled_quantity, execution_price, timestamp } => {
                info!("Order filled for instance {}: {} {} at {} (Order ID: {})", 
                      instance_id, filled_quantity, symbol, execution_price, order_id);
                
                // Notify strategy of order fill
                if let Err(e) = self.notify_strategy_order_fill(instance_id, order_id, symbol, filled_quantity, execution_price, timestamp).await {
                    error!("Failed to notify strategy {} of order fill: {}", instance_id, e);
                }
            }
            _ => {
                // Log other events
                debug!("Processed execution event: {:?}", event);
            }
        }
    }

    /// Update execution statistics
    async fn update_statistics(&self) {
        let mut stats = self.stats.write().await;
        let metadata = self.instance_metadata.read().await;

        stats.active_strategies = metadata
            .values()
            .filter(|i| i.status == InstanceStatus::Running)
            .count();

        // Update other stats...
        // This would be expanded with more comprehensive statistics
    }

    /// Create execution context for an instance
    async fn create_context_for_instance(&self, instance_id: Uuid) -> Result<StrategyContext, AppError> {
        let instance = self.get_strategy_instance(instance_id).await?;

        // This would fetch real market data, positions, etc.
        // For now, create a basic context
        StrategyContextBuilder::new()
            .strategy_id(instance_id)
            .user_id(instance.user_id)
            .symbol(instance.symbol)
            .interval(instance.interval)
            .mode(instance.mode)
            .current_time(Utc::now())
            .available_balance(rust_decimal::Decimal::from(10000)) // Mock balance
            .build()
    }

    /// Notify strategy of order fill for proper state tracking
    async fn notify_strategy_order_fill(
        &self, 
        instance_id: Uuid, 
        order_id: String, 
        symbol: String,
        filled_quantity: rust_decimal::Decimal,
        execution_price: rust_decimal::Decimal,
        timestamp: DateTime<Utc>
    ) -> Result<(), AppError> {
        // Get the strategy instance
        if let Some(strategy_arc) = {
            let instances = self.instances.read().await;
            instances.get(&instance_id).cloned()
        } {
            let mut strategy = strategy_arc.lock().await;
            
            // Create order update event
            let order_update = crate::strategies::core::OrderUpdate {
                order_id,
                symbol: symbol.clone(),
                order_type: crate::strategies::core::traits::OrderType::Market,
                status: crate::strategies::core::traits::OrderStatus::Filled,
                quantity: filled_quantity,
                price: Some(execution_price),
                filled_quantity,
                timestamp,
            };
            
            // Call the strategy's order update handler
            strategy.on_order_update(&order_update).await?;
            
            info!("Notified strategy {} of order fill: {} {} at {}", 
                  instance_id, filled_quantity, symbol, execution_price);
        }
        
        Ok(())
    }
}

// Implement Clone for Arc sharing
impl Clone for ExecutionEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            instances: self.instances.clone(),
            instance_metadata: self.instance_metadata.clone(),
            signal_executor: self.signal_executor.clone(),
            monitor: self.monitor.clone(),
            scheduler: self.scheduler.clone(),
            risk_manager: self.risk_manager.clone(),
            event_sender: self.event_sender.clone(),
            event_receiver: self.event_receiver.clone(),
            is_running: self.is_running.clone(),
            stats: self.stats.clone(),
        }
    }
}

// Add this trait for downcasting
pub trait AsAny {
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
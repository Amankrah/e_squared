use async_trait::async_trait;
use serde_json::Value;

use super::traits::{Strategy, StrategyFactory, StrategyMetadata, StrategyMode, StrategyContext};
use crate::utils::errors::AppError;

/// Generic factory for creating strategies with initialization
pub struct StrategyFactoryImpl<T> {
    metadata: StrategyMetadata,
    creator: fn() -> T,
}

impl<T> StrategyFactoryImpl<T>
where
    T: Strategy + 'static,
{
    /// Create a new factory
    pub fn new(metadata: StrategyMetadata, creator: fn() -> T) -> Self {
        Self { metadata, creator }
    }
}

impl<T> StrategyFactory for StrategyFactoryImpl<T>
where
    T: Strategy + 'static,
{
    fn create(&self) -> Box<dyn Strategy> {
        Box::new((self.creator)())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

/// Factory builder for easier strategy registration
pub struct StrategyFactoryBuilder {
    metadata: StrategyMetadata,
}

impl StrategyFactoryBuilder {
    /// Start building a factory with metadata
    pub fn new(metadata: StrategyMetadata) -> Self {
        Self { metadata }
    }

    /// Build the factory with a creator function
    pub fn with_creator<T>(self, creator: fn() -> T) -> StrategyFactoryImpl<T>
    where
        T: Strategy + 'static,
    {
        StrategyFactoryImpl::new(self.metadata, creator)
    }
}

/// Convenient macro for creating strategy factories
#[macro_export]
macro_rules! create_strategy_factory {
    ($strategy_type:ty, $metadata:expr) => {
        $crate::strategies::core::factory::StrategyFactoryBuilder::new($metadata)
            .with_creator(|| <$strategy_type>::new())
    };
}

/// Advanced factory for strategies that need custom initialization
pub struct AdvancedStrategyFactory {
    metadata: StrategyMetadata,
    creator: Box<dyn Fn() -> Box<dyn Strategy> + Send + Sync>,
}

impl AdvancedStrategyFactory {
    /// Create a new advanced factory
    pub fn new<F>(metadata: StrategyMetadata, creator: F) -> Self
    where
        F: Fn() -> Box<dyn Strategy> + Send + Sync + 'static,
    {
        Self {
            metadata,
            creator: Box::new(creator),
        }
    }
}

impl StrategyFactory for AdvancedStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        (self.creator)()
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

/// Factory for strategies that can be configured at creation time
pub struct ConfigurableStrategyFactory {
    metadata: StrategyMetadata,
    default_config: Value,
    creator: Box<dyn Fn(&Value) -> Result<Box<dyn Strategy>, AppError> + Send + Sync>,
}

impl ConfigurableStrategyFactory {
    /// Create a new configurable factory
    pub fn new<F>(
        metadata: StrategyMetadata,
        default_config: Value,
        creator: F,
    ) -> Self
    where
        F: Fn(&Value) -> Result<Box<dyn Strategy>, AppError> + Send + Sync + 'static,
    {
        Self {
            metadata,
            default_config,
            creator: Box::new(creator),
        }
    }

    /// Create strategy with custom configuration
    pub fn create_with_config(&self, config: &Value) -> Result<Box<dyn Strategy>, AppError> {
        (self.creator)(config)
    }

    /// Create strategy with default configuration
    pub fn create_with_defaults(&self) -> Result<Box<dyn Strategy>, AppError> {
        (self.creator)(&self.default_config)
    }
}

impl StrategyFactory for ConfigurableStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        self.create_with_defaults()
            .expect("Failed to create strategy with default config")
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

/// Factory manager for handling multiple strategy types
pub struct StrategyFactoryManager {
    factories: std::collections::HashMap<String, Box<dyn StrategyFactory>>,
}

impl StrategyFactoryManager {
    /// Create a new factory manager
    pub fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    /// Register a factory
    pub fn register<F>(&mut self, factory: F) -> Result<(), AppError>
    where
        F: StrategyFactory + 'static,
    {
        let strategy_id = factory.metadata().id.clone();
        self.factories.insert(strategy_id, Box::new(factory));
        Ok(())
    }

    /// Create a strategy by ID
    pub fn create(&self, strategy_id: &str) -> Result<Box<dyn Strategy>, AppError> {
        let factory = self.factories.get(strategy_id)
            .ok_or_else(|| AppError::NotFound(format!("Strategy factory not found: {}", strategy_id)))?;

        Ok(factory.create())
    }

    /// Get all available strategy IDs
    pub fn get_strategy_ids(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }

    /// Get strategy metadata
    pub fn get_metadata(&self, strategy_id: &str) -> Result<&StrategyMetadata, AppError> {
        let factory = self.factories.get(strategy_id)
            .ok_or_else(|| AppError::NotFound(format!("Strategy factory not found: {}", strategy_id)))?;

        Ok(factory.metadata())
    }
}

impl Default for StrategyFactoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for strategies that can be easily factorized
pub trait FactorizableStrategy: Strategy + Default + 'static {
    /// Get the strategy's metadata
    fn get_metadata() -> StrategyMetadata;

    /// Create a factory for this strategy type
    fn create_factory() -> StrategyFactoryImpl<Self>
    where
        Self: 'static,
    {
        StrategyFactoryImpl::new(Self::get_metadata(), Self::default)
    }
}
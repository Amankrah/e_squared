use crate::strategies::core::{register_strategy, FactorizableStrategy};
use crate::utils::errors::AppError;
use super::{SMACrossoverStrategy, SMACrossoverStrategyFactory};

/// Register the SMA Crossover strategy in the global registry
pub fn register_sma_crossover_strategy() -> Result<(), AppError> {
    let factory = SMACrossoverStrategyFactory::new();
    register_strategy(factory)?;
    tracing::info!("SMA Crossover strategy registered successfully");
    Ok(())
}

/// Register all SMA Crossover strategy variants
pub fn register_all_sma_crossover_strategies() -> Result<(), AppError> {
    // Register the main SMA Crossover strategy
    register_sma_crossover_strategy()?;

    // In the future, you could register specialized variants here
    // register_strategy(EMACorssoverStrategyFactory::new())?;
    // register_strategy(TripleSMAStrategyFactory::new())?;

    Ok(())
}

// Implement FactorizableStrategy trait for easier registration
impl FactorizableStrategy for SMACrossoverStrategy {
    fn get_metadata() -> crate::strategies::core::StrategyMetadata {
        SMACrossoverStrategy::create_metadata()
    }
}

/// Initialize SMA Crossover strategies during application startup
pub fn init_sma_crossover_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing SMA Crossover strategies...");

    match register_all_sma_crossover_strategies() {
        Ok(_) => {
            tracing::info!("All SMA Crossover strategies initialized successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to initialize SMA Crossover strategies: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{get_global_registry, create_strategy};

    #[test]
    fn test_sma_crossover_strategy_registration() {
        // Register the strategy
        assert!(register_sma_crossover_strategy().is_ok());

        // Check if it's in the registry
        let registry = get_global_registry();
        let registry = registry.read().unwrap();
        assert!(registry.contains("sma_crossover_v2"));

        // Create an instance
        drop(registry);
        let strategy = create_strategy("sma_crossover_v2");
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        let metadata = strategy.metadata();
        assert_eq!(metadata.id, "sma_crossover_v2");
        assert_eq!(metadata.name, "SMA Crossover Strategy v2");
    }

    #[test]
    fn test_sma_crossover_strategy_factory_creation() {
        let factory = SMACrossoverStrategyFactory::new();
        let metadata = factory.metadata();

        assert_eq!(metadata.id, "sma_crossover_v2");
        assert_eq!(metadata.category, crate::strategies::core::StrategyCategory::TechnicalAnalysis);
        assert_eq!(metadata.risk_level, crate::strategies::core::RiskLevel::Moderate);

        let strategy = factory.create();
        assert_eq!(strategy.metadata().id, "sma_crossover_v2");
    }
}
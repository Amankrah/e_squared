use crate::strategies::core::{register_strategy, FactorizableStrategy};
use crate::utils::errors::AppError;
use super::{GridTradingStrategy, GridTradingStrategyFactory};

/// Register the Grid Trading strategy in the global registry
pub fn register_grid_trading_strategy() -> Result<(), AppError> {
    let factory = GridTradingStrategyFactory::new();
    register_strategy(factory)?;
    tracing::info!("Grid Trading strategy registered successfully");
    Ok(())
}

/// Register all Grid Trading strategy variants
pub fn register_all_grid_trading_strategies() -> Result<(), AppError> {
    // Register the main Grid Trading strategy
    register_grid_trading_strategy()?;

    // In the future, you could register specialized variants here
    // register_strategy(DynamicGridStrategyFactory::new())?;
    // register_strategy(ArithmeticGridStrategyFactory::new())?;

    Ok(())
}

// Implement FactorizableStrategy trait for easier registration
impl FactorizableStrategy for GridTradingStrategy {
    fn get_metadata() -> crate::strategies::core::StrategyMetadata {
        GridTradingStrategy::create_metadata()
    }
}

/// Initialize Grid Trading strategies during application startup
pub fn init_grid_trading_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing Grid Trading strategies...");

    match register_all_grid_trading_strategies() {
        Ok(_) => {
            tracing::info!("All Grid Trading strategies initialized successfully");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to initialize Grid Trading strategies: {:?}", e);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::core::{get_global_registry, create_strategy, StrategyFactory};

    #[test]
    fn test_grid_trading_strategy_registration() {
        // Register the strategy
        assert!(register_grid_trading_strategy().is_ok());

        // Check if it's in the registry
        let registry = get_global_registry();
        let registry = registry.read().unwrap();
        assert!(registry.contains("grid_trading_v2"));

        // Create an instance
        drop(registry);
        let strategy = create_strategy("grid_trading_v2");
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        let metadata = strategy.metadata();
        assert_eq!(metadata.id, "grid_trading_v2");
        assert_eq!(metadata.name, "Grid Trading v2");
    }

    #[test]
    fn test_grid_trading_strategy_factory_creation() {
        let factory = GridTradingStrategyFactory::new();
        let metadata = factory.metadata();

        assert_eq!(metadata.id, "grid_trading_v2");
        assert_eq!(metadata.category, crate::strategies::core::StrategyCategory::GridTrading);
        assert_eq!(metadata.risk_level, crate::strategies::core::RiskLevel::Moderate);

        let strategy = factory.create();
        assert_eq!(strategy.metadata().id, "grid_trading_v2");
    }
}
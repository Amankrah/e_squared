pub mod indicators;
pub mod core;
pub mod implementations;

// Re-export core types and traits for convenience
pub use core::*;

// Initialize all strategies
use crate::utils::errors::AppError;

/// Initialize all available strategies
pub fn init_all_strategies() -> Result<(), AppError> {
    tracing::info!("Initializing trading strategies...");

    // Initialize DCA strategies
    implementations::dca::init_dca_strategies()?;

    // TODO: Initialize other strategy types as they are implemented
    // implementations::rsi::init_rsi_strategies()?;
    // implementations::sma_crossover::init_sma_strategies()?;
    // implementations::macd::init_macd_strategies()?;
    // implementations::grid_trading::init_grid_strategies()?;

    tracing::info!("All trading strategies initialized successfully");
    Ok(())
}
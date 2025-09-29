use crate::strategies::core::{Strategy, StrategyFactory, StrategyMetadata};
use super::RSIStrategy;

/// Factory for creating RSI strategy instances
pub struct RSIStrategyFactory {
    metadata: StrategyMetadata,
}

impl RSIStrategyFactory {
    /// Create a new RSI strategy factory
    pub fn new() -> Self {
        Self {
            metadata: RSIStrategy::create_metadata(),
        }
    }
}

impl StrategyFactory for RSIStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        Box::new(RSIStrategy::new())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

impl Default for RSIStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}
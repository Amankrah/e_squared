use crate::strategies::core::{Strategy, StrategyFactory, StrategyMetadata};
use super::MACDStrategy;

/// Factory for creating MACD strategy instances
pub struct MACDStrategyFactory {
    metadata: StrategyMetadata,
}

impl MACDStrategyFactory {
    /// Create a new MACD strategy factory
    pub fn new() -> Self {
        Self {
            metadata: MACDStrategy::create_metadata(),
        }
    }
}

impl StrategyFactory for MACDStrategyFactory {
    fn create(&self) -> Box<dyn Strategy> {
        Box::new(MACDStrategy::new())
    }

    fn metadata(&self) -> &StrategyMetadata {
        &self.metadata
    }
}

impl Default for MACDStrategyFactory {
    fn default() -> Self {
        Self::new()
    }
}
pub mod market_data_service;
pub mod dca_execution_engine;
pub mod dxy_service;
pub mod market_indicators_service;
pub mod stock_data_service;
// Removed legacy strategy_templates - using new modular system

pub use market_data_service::*;
pub use dca_execution_engine::*;
pub use dxy_service::*;
pub use market_indicators_service::*;
pub use stock_data_service::*;
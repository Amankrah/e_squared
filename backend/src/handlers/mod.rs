pub mod auth;
pub mod user_profile;
pub mod two_factor;
pub mod session_management;
pub mod exchange_management;
pub mod dca_strategy_management;
pub mod rsi_strategy_management;
pub mod macd_strategy_management;
pub mod sma_crossover_strategy_management;
pub mod grid_trading_strategy_management;
// Removed legacy strategy_templates_handler - using new modular system
pub use auth::*;
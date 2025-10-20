use super::errors::DexError;
use super::traits::DexConnector;
use super::{WalletCredentials, DEX};
use std::sync::Arc;

/// DEX connector factory
pub struct DexFactory;

impl DexFactory {
    /// Create a DEX connector based on the DEX type
    pub fn create(
        dex: DEX,
        credentials: WalletCredentials,
    ) -> Result<Arc<dyn DexConnector>, DexError> {
        match dex {
            DEX::Uniswap => {
                let connector = super::uniswap::UniswapConnector::new(credentials)?;
                Ok(Arc::new(connector))
            }
            DEX::PancakeSwap => {
                let connector = super::pancakeswap::PancakeSwapConnector::new(credentials)?;
                Ok(Arc::new(connector))
            }
            DEX::Raydium => {
                let connector = super::raydium::RaydiumConnector::new(credentials)?;
                Ok(Arc::new(connector))
            }
            DEX::Jupiter => {
                let connector = super::jupiter::JupiterConnector::new(credentials)?;
                Ok(Arc::new(connector))
            }
        }
    }
}

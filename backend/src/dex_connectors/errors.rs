use thiserror::Error;

#[derive(Error, Debug)]
pub enum DexError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Invalid wallet credentials: {0}")]
    InvalidCredentials(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Slippage too high: {0}")]
    SlippageTooHigh(String),

    #[error("Gas estimation failed: {0}")]
    GasEstimationFailed(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Contract interaction failed: {0}")]
    ContractError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<reqwest::Error> for DexError {
    fn from(err: reqwest::Error) -> Self {
        DexError::NetworkError(err.to_string())
    }
}

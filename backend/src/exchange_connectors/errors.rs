use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExchangeError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("API key not found or invalid")]
    InvalidApiKey,

    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Exchange API error: {0}")]
    ApiError(String),

    #[error("Not supported by exchange: {0}")]
    NotSupported(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Connection timeout")]
    Timeout,

    #[error("Exchange is under maintenance")]
    Maintenance,

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<reqwest::Error> for ExchangeError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ExchangeError::Timeout
        } else if err.is_connect() {
            ExchangeError::NetworkError(format!("Connection failed: {}", err))
        } else {
            ExchangeError::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for ExchangeError {
    fn from(err: serde_json::Error) -> Self {
        ExchangeError::ParseError(err.to_string())
    }
}

impl From<std::io::Error> for ExchangeError {
    fn from(err: std::io::Error) -> Self {
        ExchangeError::InternalError(err.to_string())
    }
}
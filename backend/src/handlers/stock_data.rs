use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::utils::errors::AppError;
use crate::services::StockDataService;

#[derive(Debug, Serialize)]
pub struct StockPriceResponse {
    pub symbol: String,
    pub price: f64,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoricalDataQuery {
    pub outputsize: Option<String>,
}

/// Get current stock price
pub async fn get_stock_price(
    symbol: web::Path<String>,
    stock_service: web::Data<StockDataService>,
) -> Result<HttpResponse, AppError> {
    let symbol = symbol.into_inner().to_uppercase();

    info!("Fetching current price for stock: {}", symbol);

    let stock_price = stock_service.get_current_price(&symbol).await?;

    info!("Stock {} current price: ${}", symbol, stock_price.price);

    Ok(HttpResponse::Ok().json(stock_price))
}

/// Get historical stock data
pub async fn get_stock_historical(
    symbol: web::Path<String>,
    query: web::Query<HistoricalDataQuery>,
    stock_service: web::Data<StockDataService>,
) -> Result<HttpResponse, AppError> {
    let symbol = symbol.into_inner().to_uppercase();

    info!("Fetching historical data for stock: {}", symbol);

    let outputsize = query.outputsize.as_deref();
    let historical_data = stock_service.get_historical_data(&symbol, outputsize).await?;

    info!("Retrieved {} historical data points for {}", historical_data.data.len(), symbol);

    Ok(HttpResponse::Ok().json(historical_data))
}

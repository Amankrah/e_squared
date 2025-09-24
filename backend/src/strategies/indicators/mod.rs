// New modular indicator system
pub mod core;
pub mod moving_averages;
pub mod momentum;
pub mod service;
pub mod integration;
pub mod examples;

// Re-export main components for easy access
pub use core::*;
pub use service::{IndicatorService, IndicatorServiceConfig};
pub use integration::{IndicatorContext, utils};

// Legacy functions for backward compatibility
use rust_decimal::{Decimal, prelude::*};
use crate::exchange_connectors::Kline;

/// Simple Moving Average
pub fn sma(data: &[Kline], period: usize) -> Option<Decimal> {
    if data.len() < period {
        return None;
    }

    let sum = data
        .iter()
        .rev()
        .take(period)
        .map(|k| k.close)
        .sum::<Decimal>();

    Some(sum / Decimal::from(period))
}

/// Exponential Moving Average
pub fn ema(data: &[Kline], period: usize) -> Option<Decimal> {
    if data.is_empty() {
        return None;
    }

    if data.len() < period {
        // Use SMA for the first calculation
        return sma(data, data.len());
    }

    let multiplier = Decimal::from(2) / Decimal::from(period + 1);

    // Start with SMA of the first 'period' values
    let initial_sma = data[..period]
        .iter()
        .map(|k| k.close)
        .sum::<Decimal>() / Decimal::from(period);

    let mut ema_value = initial_sma;

    // Calculate EMA for the rest of the values
    for kline in data.iter().skip(period) {
        ema_value = (kline.close * multiplier) + (ema_value * (Decimal::ONE - multiplier));
    }

    Some(ema_value)
}

/// Relative Strength Index
pub fn rsi(data: &[Kline], period: usize) -> Option<Decimal> {
    if data.len() < period + 1 {
        return None;
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calculate price changes
    for window in data.windows(2) {
        let change = window[1].close - window[0].close;
        if change > Decimal::ZERO {
            gains.push(change);
            losses.push(Decimal::ZERO);
        } else {
            gains.push(Decimal::ZERO);
            losses.push(change.abs());
        }
    }

    if gains.len() < period {
        return None;
    }

    // Calculate average gains and losses
    let avg_gain = gains
        .iter()
        .rev()
        .take(period)
        .sum::<Decimal>() / Decimal::from(period);

    let avg_loss = losses
        .iter()
        .rev()
        .take(period)
        .sum::<Decimal>() / Decimal::from(period);

    if avg_loss == Decimal::ZERO {
        return Some(Decimal::from(100)); // RSI = 100 when there are no losses
    }

    let rs = avg_gain / avg_loss;
    let rsi = Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs));

    Some(rsi)
}

/// Bollinger Bands
#[derive(Debug, Clone)]
pub struct BollingerBands {
    pub upper: Decimal,
    pub middle: Decimal,
    pub lower: Decimal,
}

pub fn bollinger_bands(data: &[Kline], period: usize, std_dev_multiplier: Decimal) -> Option<BollingerBands> {
    let middle = sma(data, period)?;

    if data.len() < period {
        return None;
    }

    // Calculate standard deviation
    let recent_closes: Vec<Decimal> = data
        .iter()
        .rev()
        .take(period)
        .map(|k| k.close)
        .collect();

    let variance = recent_closes
        .iter()
        .map(|&price| {
            let diff = price - middle;
            diff * diff // Use multiplication instead of powi
        })
        .sum::<Decimal>() / Decimal::from(period);

    // Simplified square root approximation
    let std_dev = variance; // For now, just use variance as approximation

    let upper = middle + (std_dev * std_dev_multiplier);
    let lower = middle - (std_dev * std_dev_multiplier);

    Some(BollingerBands {
        upper,
        middle,
        lower,
    })
}

/// MACD (Moving Average Convergence Divergence)
#[derive(Debug, Clone)]
pub struct MACD {
    pub macd_line: Decimal,
    pub signal_line: Decimal,
    pub histogram: Decimal,
}

pub fn macd(data: &[Kline], fast_period: usize, slow_period: usize, signal_period: usize) -> Option<MACD> {
    let fast_ema = ema(data, fast_period)?;
    let slow_ema = ema(data, slow_period)?;

    let macd_line = fast_ema - slow_ema;

    // For signal line, we would need historical MACD values
    // This is a simplified implementation
    let signal_line = macd_line; // In practice, this should be EMA of MACD line

    let histogram = macd_line - signal_line;

    Some(MACD {
        macd_line,
        signal_line,
        histogram,
    })
}

/// Stochastic Oscillator
#[derive(Debug, Clone)]
pub struct Stochastic {
    pub k_percent: Decimal,
    pub d_percent: Decimal,
}

pub fn stochastic(data: &[Kline], k_period: usize, d_period: usize) -> Option<Stochastic> {
    if data.len() < k_period {
        return None;
    }

    let recent_klines: Vec<&Kline> = data.iter().rev().take(k_period).collect();

    let highest_high = recent_klines.iter().map(|k| k.high).max()?;
    let lowest_low = recent_klines.iter().map(|k| k.low).min()?;
    let current_close = data.last()?.close;

    let k_percent = if highest_high != lowest_low {
        ((current_close - lowest_low) / (highest_high - lowest_low)) * Decimal::from(100)
    } else {
        Decimal::from(50) // Default when high == low
    };

    // For D%, we would need historical %K values
    // This is a simplified implementation
    let d_percent = k_percent; // In practice, this should be SMA of %K

    Some(Stochastic {
        k_percent,
        d_percent,
    })
}

/// Average True Range
pub fn atr(data: &[Kline], period: usize) -> Option<Decimal> {
    if data.len() < period + 1 {
        return None;
    }

    let mut true_ranges = Vec::new();

    for window in data.windows(2) {
        let prev_close = window[0].close;
        let current_high = window[1].high;
        let current_low = window[1].low;

        let tr1 = current_high - current_low;
        let tr2 = (current_high - prev_close).abs();
        let tr3 = (current_low - prev_close).abs();

        let true_range = tr1.max(tr2).max(tr3);
        true_ranges.push(true_range);
    }

    if true_ranges.len() < period {
        return None;
    }

    let atr = true_ranges
        .iter()
        .rev()
        .take(period)
        .sum::<Decimal>() / Decimal::from(period);

    Some(atr)
}

/// Volume Weighted Average Price
pub fn vwap(data: &[Kline]) -> Option<Decimal> {
    if data.is_empty() {
        return None;
    }

    let mut total_volume = Decimal::ZERO;
    let mut total_price_volume = Decimal::ZERO;

    for kline in data {
        let typical_price = (kline.high + kline.low + kline.close) / Decimal::from(3);
        total_price_volume += typical_price * kline.volume;
        total_volume += kline.volume;
    }

    if total_volume > Decimal::ZERO {
        Some(total_price_volume / total_volume)
    } else {
        None
    }
}

/// Williams %R
pub fn williams_r(data: &[Kline], period: usize) -> Option<Decimal> {
    if data.len() < period {
        return None;
    }

    let recent_klines: Vec<&Kline> = data.iter().rev().take(period).collect();

    let highest_high = recent_klines.iter().map(|k| k.high).max()?;
    let lowest_low = recent_klines.iter().map(|k| k.low).min()?;
    let current_close = data.last()?.close;

    if highest_high != lowest_low {
        let williams_r = ((highest_high - current_close) / (highest_high - lowest_low)) * Decimal::from(-100);
        Some(williams_r)
    } else {
        Some(Decimal::from(-50)) // Default when high == low
    }
}
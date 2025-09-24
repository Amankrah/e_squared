use async_trait::async_trait;
use rust_decimal::{Decimal, prelude::*};
use chrono::Utc;
use std::collections::VecDeque;

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::core::*;

/// Relative Strength Index with Wilder's smoothing
#[derive(Debug, Clone)]
pub struct RSI {
    period: usize,
    avg_gain: Option<Decimal>,
    avg_loss: Option<Decimal>,
    prev_close: Option<Decimal>,
    sample_count: usize,
    last_value: Option<IndicatorValue>,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            avg_gain: None,
            avg_loss: None,
            prev_close: None,
            sample_count: 0,
            last_value: None,
        }
    }

    fn calculate(&self) -> Option<Decimal> {
        match (self.avg_gain, self.avg_loss) {
            (Some(avg_gain), Some(avg_loss)) => {
                if avg_loss == Decimal::ZERO {
                    Some(Decimal::from(100))
                } else {
                    let rs = avg_gain / avg_loss;
                    Some(Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs)))
                }
            }
            _ => None,
        }
    }
}

#[async_trait]
impl Indicator for RSI {
    fn name(&self) -> String {
        format!("RSI({})", self.period)
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        if let Some(prev_close) = self.prev_close {
            let change = kline.close - prev_close;
            let (gain, loss) = if change > Decimal::ZERO {
                (change, Decimal::ZERO)
            } else {
                (Decimal::ZERO, change.abs())
            };

            // Update averages using Wilder's smoothing
            match (self.avg_gain, self.avg_loss) {
                (Some(prev_avg_gain), Some(prev_avg_loss)) => {
                    // Wilder's smoothing: new_avg = (prev_avg * (n-1) + new_value) / n
                    let n = Decimal::from(self.period);
                    self.avg_gain = Some((prev_avg_gain * (n - Decimal::ONE) + gain) / n);
                    self.avg_loss = Some((prev_avg_loss * (n - Decimal::ONE) + loss) / n);
                }
                _ => {
                    // Initial average
                    self.avg_gain = Some(gain);
                    self.avg_loss = Some(loss);
                }
            }

            self.sample_count += 1;

            if let Some(rsi_value) = self.calculate() {
                self.last_value = Some(IndicatorValue {
                    value: IndicatorResult::Single(rsi_value),
                    timestamp: Utc::now(),
                    confidence: if self.sample_count >= self.period {
                        Decimal::ONE
                    } else {
                        Decimal::from(self.sample_count) / Decimal::from(self.period)
                    },
                });
            }
        }

        self.prev_close = Some(kline.close);
        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.avg_gain = None;
        self.avg_loss = None;
        self.prev_close = None;
        self.sample_count = 0;
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.sample_count >= self.period
    }

    fn min_periods(&self) -> usize {
        self.period + 1
    }

    fn clone_box(&self) -> Box<dyn Indicator> {
        Box::new(self.clone())
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "period": self.period
        })
    }
}

/// MACD with proper signal line calculation
#[derive(Debug, Clone)]
pub struct MACD {
    fast_ema: EMA,
    slow_ema: EMA,
    signal_ema: EMA,
    macd_history: VecDeque<Decimal>,
    last_value: Option<IndicatorValue>,
}

use super::moving_averages::EMA;

impl MACD {
    pub fn new(fast_period: usize, slow_period: usize, signal_period: usize) -> Self {
        Self {
            fast_ema: EMA::new(fast_period),
            slow_ema: EMA::new(slow_period),
            signal_ema: EMA::new(signal_period),
            macd_history: VecDeque::with_capacity(signal_period),
            last_value: None,
        }
    }
}

#[async_trait]
impl Indicator for MACD {
    fn name(&self) -> String {
        format!("MACD({},{},{})",
            self.fast_ema.period,
            self.slow_ema.period,
            self.signal_ema.period
        )
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        // Update EMAs
        self.fast_ema.update(kline).await?;
        self.slow_ema.update(kline).await?;

        // Calculate MACD line if both EMAs are ready
        if let (Some(fast_val), Some(slow_val)) = (self.fast_ema.value(), self.slow_ema.value()) {
            let macd_line = match (fast_val.value, slow_val.value) {
                (IndicatorResult::Single(fast), IndicatorResult::Single(slow)) => fast - slow,
                _ => return Ok(()),
            };

            // Create a synthetic kline for signal line EMA
            let synthetic_kline = Kline {
                open_time: kline.open_time,
                close_time: kline.close_time,
                open: macd_line,
                high: macd_line,
                low: macd_line,
                close: macd_line,
                volume: Decimal::ZERO,
                quote_asset_volume: Decimal::ZERO,
                number_of_trades: 0,
                taker_buy_base_asset_volume: Decimal::ZERO,
                taker_buy_quote_asset_volume: Decimal::ZERO,
            };

            // Update signal line
            self.signal_ema.update(&synthetic_kline).await?;

            // Get signal line value
            let signal_line = if let Some(signal_val) = self.signal_ema.value() {
                match signal_val.value {
                    IndicatorResult::Single(val) => val,
                    _ => macd_line,
                }
            } else {
                macd_line
            };

            let histogram = macd_line - signal_line;

            self.last_value = Some(IndicatorValue {
                value: IndicatorResult::MACD {
                    macd_line,
                    signal_line,
                    histogram,
                },
                timestamp: Utc::now(),
                confidence: if self.slow_ema.is_ready() && self.signal_ema.is_ready() {
                    Decimal::ONE
                } else {
                    Decimal::new(5, 1) // 0.5
                },
            });
        }

        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.fast_ema.reset();
        self.slow_ema.reset();
        self.signal_ema.reset();
        self.macd_history.clear();
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.slow_ema.is_ready() && self.signal_ema.is_ready()
    }

    fn min_periods(&self) -> usize {
        self.slow_ema.min_periods() + self.signal_ema.min_periods()
    }

    fn clone_box(&self) -> Box<dyn Indicator> {
        Box::new(self.clone())
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "fast_period": self.fast_ema.period,
            "slow_period": self.slow_ema.period,
            "signal_period": self.signal_ema.period
        })
    }
}

/// Stochastic Oscillator with proper %D calculation
#[derive(Debug, Clone)]
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
    high_buffer: VecDeque<Decimal>,
    low_buffer: VecDeque<Decimal>,
    close_buffer: VecDeque<Decimal>,
    k_values: VecDeque<Decimal>,
    last_value: Option<IndicatorValue>,
}

impl Stochastic {
    pub fn new(k_period: usize, d_period: usize) -> Self {
        Self {
            k_period,
            d_period,
            high_buffer: VecDeque::with_capacity(k_period),
            low_buffer: VecDeque::with_capacity(k_period),
            close_buffer: VecDeque::with_capacity(k_period),
            k_values: VecDeque::with_capacity(d_period),
            last_value: None,
        }
    }

    fn calculate_k(&self) -> Option<Decimal> {
        if self.high_buffer.len() < self.k_period {
            return None;
        }

        let highest = self.high_buffer.iter().max()?;
        let lowest = self.low_buffer.iter().min()?;
        let current_close = self.close_buffer.back()?;

        if highest != lowest {
            Some(((current_close - lowest) / (highest - lowest)) * Decimal::from(100))
        } else {
            Some(Decimal::from(50))
        }
    }

    fn calculate_d(&self) -> Option<Decimal> {
        if self.k_values.len() >= self.d_period {
            let sum: Decimal = self.k_values.iter().sum();
            Some(sum / Decimal::from(self.d_period))
        } else if !self.k_values.is_empty() {
            let sum: Decimal = self.k_values.iter().sum();
            Some(sum / Decimal::from(self.k_values.len()))
        } else {
            None
        }
    }
}

#[async_trait]
impl Indicator for Stochastic {
    fn name(&self) -> String {
        format!("Stochastic({},{})", self.k_period, self.d_period)
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        // Update buffers
        self.high_buffer.push_back(kline.high);
        self.low_buffer.push_back(kline.low);
        self.close_buffer.push_back(kline.close);

        // Maintain buffer size
        if self.high_buffer.len() > self.k_period {
            self.high_buffer.pop_front();
            self.low_buffer.pop_front();
            self.close_buffer.pop_front();
        }

        // Calculate %K
        if let Some(k_value) = self.calculate_k() {
            self.k_values.push_back(k_value);

            // Maintain k_values buffer size
            if self.k_values.len() > self.d_period {
                self.k_values.pop_front();
            }

            // Calculate %D
            if let Some(d_value) = self.calculate_d() {
                self.last_value = Some(IndicatorValue {
                    value: IndicatorResult::Stochastic {
                        k_percent: k_value,
                        d_percent: d_value,
                    },
                    timestamp: Utc::now(),
                    confidence: if self.is_ready() {
                        Decimal::ONE
                    } else {
                        Decimal::new(7, 1) // 0.7
                    },
                });
            }
        }

        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.high_buffer.clear();
        self.low_buffer.clear();
        self.close_buffer.clear();
        self.k_values.clear();
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.high_buffer.len() >= self.k_period && self.k_values.len() >= self.d_period
    }

    fn min_periods(&self) -> usize {
        self.k_period + self.d_period - 1
    }

    fn clone_box(&self) -> Box<dyn Indicator> {
        Box::new(self.clone())
    }

    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "k_period": self.k_period,
            "d_period": self.d_period
        })
    }
}
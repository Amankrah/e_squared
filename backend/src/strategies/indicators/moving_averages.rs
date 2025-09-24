use async_trait::async_trait;
use rust_decimal::{Decimal, prelude::*};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use crate::exchange_connectors::Kline;
use crate::utils::errors::AppError;
use super::core::*;

/// Simple Moving Average with state
#[derive(Debug, Clone)]
pub struct SMA {
    period: usize,
    buffer: VecDeque<Decimal>,
    current_sum: Decimal,
    last_value: Option<IndicatorValue>,
}

impl SMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            buffer: VecDeque::with_capacity(period),
            current_sum: Decimal::ZERO,
            last_value: None,
        }
    }

    fn calculate(&mut self) -> Decimal {
        if self.buffer.len() >= self.period {
            self.current_sum / Decimal::from(self.period)
        } else if !self.buffer.is_empty() {
            self.current_sum / Decimal::from(self.buffer.len())
        } else {
            Decimal::ZERO
        }
    }
}

#[async_trait]
impl Indicator for SMA {
    fn name(&self) -> String {
        format!("SMA({})", self.period)
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        // Add new value to buffer
        self.buffer.push_back(kline.close);
        self.current_sum += kline.close;

        // Remove old value if buffer is full
        if self.buffer.len() > self.period {
            if let Some(old_value) = self.buffer.pop_front() {
                self.current_sum -= old_value;
            }
        }

        // Calculate and store new value
        let sma_value = self.calculate();
        self.last_value = Some(IndicatorValue {
            value: IndicatorResult::Single(sma_value),
            timestamp: Utc::now(),
            confidence: if self.buffer.len() >= self.period {
                Decimal::ONE
            } else {
                Decimal::from(self.buffer.len()) / Decimal::from(self.period)
            },
        });

        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.current_sum = Decimal::ZERO;
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.buffer.len() >= self.period
    }

    fn min_periods(&self) -> usize {
        self.period
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

/// Exponential Moving Average with state
#[derive(Debug, Clone)]
pub struct EMA {
    pub period: usize,
    multiplier: Decimal,
    current_ema: Option<Decimal>,
    sample_count: usize,
    last_value: Option<IndicatorValue>,
}

impl EMA {
    pub fn new(period: usize) -> Self {
        let multiplier = Decimal::from(2) / Decimal::from(period + 1);
        Self {
            period,
            multiplier,
            current_ema: None,
            sample_count: 0,
            last_value: None,
        }
    }
}

#[async_trait]
impl Indicator for EMA {
    fn name(&self) -> String {
        format!("EMA({})", self.period)
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        self.sample_count += 1;

        let new_ema = match self.current_ema {
            Some(prev_ema) => {
                // EMA = Price * multiplier + Previous EMA * (1 - multiplier)
                (kline.close * self.multiplier) + (prev_ema * (Decimal::ONE - self.multiplier))
            }
            None => {
                // First value: use the price as initial EMA
                kline.close
            }
        };

        self.current_ema = Some(new_ema);
        self.last_value = Some(IndicatorValue {
            value: IndicatorResult::Single(new_ema),
            timestamp: Utc::now(),
            confidence: if self.sample_count >= self.period {
                Decimal::ONE
            } else {
                Decimal::from(self.sample_count) / Decimal::from(self.period)
            },
        });

        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.current_ema = None;
        self.sample_count = 0;
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.sample_count >= self.period
    }

    fn min_periods(&self) -> usize {
        self.period
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

/// Weighted Moving Average with state
#[derive(Debug, Clone)]
pub struct WMA {
    period: usize,
    buffer: VecDeque<Decimal>,
    weight_sum: Decimal,
    last_value: Option<IndicatorValue>,
}

impl WMA {
    pub fn new(period: usize) -> Self {
        // Calculate weight sum: 1 + 2 + 3 + ... + period
        let weight_sum = Decimal::from(period * (period + 1) / 2);
        Self {
            period,
            buffer: VecDeque::with_capacity(period),
            weight_sum,
            last_value: None,
        }
    }

    fn calculate(&self) -> Decimal {
        if self.buffer.is_empty() {
            return Decimal::ZERO;
        }

        let mut weighted_sum = Decimal::ZERO;
        let mut weight = Decimal::from(1);

        for value in self.buffer.iter() {
            weighted_sum += *value * weight;
            weight += Decimal::ONE;
        }

        weighted_sum / self.weight_sum
    }
}

#[async_trait]
impl Indicator for WMA {
    fn name(&self) -> String {
        format!("WMA({})", self.period)
    }

    fn value(&self) -> Option<IndicatorValue> {
        self.last_value.clone()
    }

    async fn update(&mut self, kline: &Kline) -> Result<(), AppError> {
        self.buffer.push_back(kline.close);

        if self.buffer.len() > self.period {
            self.buffer.pop_front();
        }

        let wma_value = self.calculate();
        self.last_value = Some(IndicatorValue {
            value: IndicatorResult::Single(wma_value),
            timestamp: Utc::now(),
            confidence: if self.buffer.len() >= self.period {
                Decimal::ONE
            } else {
                Decimal::from(self.buffer.len()) / Decimal::from(self.period)
            },
        });

        Ok(())
    }

    async fn update_batch(&mut self, klines: &[Kline]) -> Result<(), AppError> {
        for kline in klines {
            self.update(kline).await?;
        }
        Ok(())
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.last_value = None;
    }

    fn is_ready(&self) -> bool {
        self.buffer.len() >= self.period
    }

    fn min_periods(&self) -> usize {
        self.period
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
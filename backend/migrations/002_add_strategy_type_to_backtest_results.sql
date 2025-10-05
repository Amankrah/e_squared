-- Add strategy_type column to backtest_results table
-- This tracks the sub-strategy type (e.g., "Simple", "RSIBased", "VolatilityBased" for DCA)

ALTER TABLE backtest_results ADD COLUMN strategy_type TEXT;

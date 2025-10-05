-- Add total_invested column to backtest_results table
-- This tracks the total amount actually invested during backtesting (important for DCA strategies)

ALTER TABLE backtest_results ADD COLUMN total_invested REAL NOT NULL DEFAULT 0;

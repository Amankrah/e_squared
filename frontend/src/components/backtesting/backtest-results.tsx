"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { 
  TrendingUp, 
  TrendingDown, 
  Target,
  Award,
  Download,
  Share2
} from "lucide-react"
import { BacktestResult, StrategyInfo } from "@/lib/api"

// Extended interface for results that include detailed data
interface ExtendedBacktestResult extends BacktestResult {
  equity_curve?: Array<{
    timestamp: string
    portfolio_value: string
    drawdown: string
    cumulative_return: string
  }>
  trades_data?: Array<{
    timestamp: string
    trade_type: 'Buy' | 'Sell'
    price: string
    quantity: string
    total_value: string
    portfolio_value: string
    balance_remaining: string
    reason: string
    pnl?: string
    pnl_percentage?: string
    entry_date?: string
    exit_date?: string
    entry_price?: string
    exit_price?: string
    side?: 'buy' | 'sell'
    duration_hours?: number
    signal_reason?: string
  }>
}
import { formatCurrency, formatPercentage } from "@/lib/strategies"
import { cn } from "@/lib/utils"

interface BacktestResultsProps {
  results: ExtendedBacktestResult
  strategyInfo: StrategyInfo
  className?: string
}

export function BacktestResults({
  results,
  strategyInfo,
  className
}: BacktestResultsProps) {
  const [selectedTab, setSelectedTab] = useState('overview')

  const isProfit = parseFloat(results.total_return) >= 0
  const returnColor = isProfit ? 'text-green-400' : 'text-red-400'

  // Check if this is a DCA strategy
  const isDCA = results.strategy_name?.toLowerCase().includes('dca') || false

  // Get total_invested from metrics if available (for engine results)
  const totalInvested = (results as any).total_invested || null

  const getPerformanceRating = () => {
    const totalReturn = parseFloat(results.total_return_percentage)
    const sharpeRatio = results.sharpe_ratio ? parseFloat(results.sharpe_ratio) : 0
    const winRate = parseFloat(results.win_rate)

    // Simple scoring system
    let score = 0
    if (totalReturn > 20) score += 3
    else if (totalReturn > 10) score += 2
    else if (totalReturn > 0) score += 1

    if (sharpeRatio > 1.5) score += 3
    else if (sharpeRatio > 1.0) score += 2
    else if (sharpeRatio > 0.5) score += 1

    if (winRate > 60) score += 2
    else if (winRate > 50) score += 1

    if (score >= 7) return { rating: 'Excellent', color: 'text-green-400', bgColor: 'bg-green-500/10 border-green-500/20' }
    if (score >= 5) return { rating: 'Good', color: 'text-blue-400', bgColor: 'bg-blue-500/10 border-blue-500/20' }
    if (score >= 3) return { rating: 'Fair', color: 'text-yellow-400', bgColor: 'bg-yellow-500/10 border-yellow-500/20' }
    return { rating: 'Poor', color: 'text-red-400', bgColor: 'bg-red-500/10 border-red-500/20' }
  }

  const performanceRating = getPerformanceRating()

  // Create simple chart data (you could integrate with a charting library like Chart.js or Recharts)
  const createEquityCurve = () => {
    // Use total_invested as baseline for DCA, initial_balance for others
    const baseAmount = isDCA && totalInvested
      ? parseFloat(totalInvested)
      : parseFloat(results.initial_balance)

    // Check if equity_curve data exists and is an array
    if (!results.equity_curve || !Array.isArray(results.equity_curve) || results.equity_curve.length === 0) {
      // Fallback: create a simple line from base to final balance
      const finalBalance = parseFloat(results.final_balance)
      const returnPct = ((finalBalance - baseAmount) / baseAmount) * 100

      // Create a smooth curve instead of straight line
      const points = []
      for (let i = 0; i <= 20; i++) {
        const progress = i / 20
        // Simulate gradual growth
        const value = baseAmount + (finalBalance - baseAmount) * progress
        points.push({
          x: progress * 100,
          y: ((value - baseAmount) / baseAmount) * 100,
          rawValue: value
        })
      }

      const maxY = Math.max(...points.map(p => p.y))
      const minY = Math.min(...points.map(p => p.y))
      const range = Math.max(maxY - minY, 1) // Avoid division by zero

      return points.map(p => ({
        ...p,
        normalizedY: range === 0 ? 50 : ((p.y - minY) / range) * 80 + 10
      }))
    }

    // Use actual equity curve data
    const points = results.equity_curve.map((dataPoint, index) => {
      const portfolioValue = parseFloat(
        dataPoint.portfolio_value ||
        ('value' in dataPoint ? (dataPoint as { value: string }).value : results.final_balance)
      )
      return {
        x: (index / (results.equity_curve!.length - 1)) * 100,
        y: ((portfolioValue - baseAmount) / baseAmount) * 100,
        rawValue: portfolioValue
      }
    })

    const maxY = Math.max(...points.map(p => p.y))
    const minY = Math.min(...points.map(p => p.y))
    const range = Math.max(maxY - minY, 1) // Avoid division by zero

    return points.map(p => ({
      ...p,
      normalizedY: range === 0 ? 50 : ((p.y - minY) / range) * 80 + 10
    }))
  }

  const equityCurve = createEquityCurve()

  return (
    <div className={cn("space-y-6", className)}>
      {/* Performance Overview */}
      <Card className={cn(
        "bg-gradient-to-br backdrop-blur-xl border border-white/10",
        strategyInfo.color
      )}>
        <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
        
        <div className="relative z-10">
          <CardHeader className="text-center space-y-4">
            <div className="flex items-center justify-center space-x-3">
              <div className="text-3xl">{strategyInfo.icon}</div>
              <div>
                <CardTitle className="text-2xl font-bold text-white/90">
                  Backtest Results: {strategyInfo.name}
                </CardTitle>
                <CardDescription className="text-white/60">
                  {results.symbol} • {results.strategy_type || 'Standard'} Strategy
                </CardDescription>
                <CardDescription className="text-white/50 text-sm">
                  {new Date(results.start_date).toLocaleDateString()} - {new Date(results.end_date).toLocaleDateString()}
                  ({Math.round((new Date(results.end_date).getTime() - new Date(results.start_date).getTime()) / (1000 * 60 * 60 * 24))} days)
                </CardDescription>
              </div>
            </div>
            
            {/* Performance Rating */}
            <div className={cn(
              "inline-flex items-center space-x-2 px-4 py-2 rounded-full border",
              performanceRating.bgColor
            )}>
              <Award className="h-4 w-4" />
              <span className={cn("font-medium", performanceRating.color)}>
                {performanceRating.rating} Performance
              </span>
            </div>
          </CardHeader>

          <CardContent>
            {/* Key Metrics Grid */}
            <div className={cn(
              "grid gap-4 mb-6",
              isDCA && totalInvested ? "grid-cols-2 md:grid-cols-5" : "grid-cols-2 md:grid-cols-4"
            )}>
              {/* For DCA: Show Total Invested first */}
              {isDCA && totalInvested && (
                <div className="text-center p-4 bg-gradient-to-br from-cyan-500/10 to-blue-500/10 rounded-lg border border-cyan-500/20">
                  <div className="text-2xl font-bold text-cyan-400">
                    {formatCurrency(totalInvested)}
                  </div>
                  <div className="text-sm text-white/60 mt-1">Total Invested</div>
                  <div className="text-xs text-white/50">Capital deployed</div>
                </div>
              )}

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className={cn("text-2xl font-bold", returnColor)}>
                  {formatCurrency(results.total_return)}
                </div>
                <div className="text-sm text-white/60 mt-1">Total Return</div>
                <div className={cn("text-sm font-medium", returnColor)}>
                  ({formatPercentage(results.total_return_percentage)})
                </div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-white/90">
                  {results.sharpe_ratio ? parseFloat(results.sharpe_ratio).toFixed(2) : 'N/A'}
                </div>
                <div className="text-sm text-white/60 mt-1">Sharpe Ratio</div>
                <div className="text-xs text-white/50">Risk-adjusted return</div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-red-400">
                  {formatPercentage(results.max_drawdown_percentage)}
                </div>
                <div className="text-sm text-white/60 mt-1">Max Drawdown</div>
                <div className="text-xs text-white/50">Worst loss period</div>
              </div>

              <div className="text-center p-4 bg-white/5 rounded-lg">
                <div className="text-2xl font-bold text-blue-400">
                  {formatPercentage(results.win_rate)}
                </div>
                <div className="text-sm text-white/60 mt-1">Win Rate</div>
                <div className="text-xs text-white/50">
                  {isDCA
                    ? `${results.winning_trades} win / ${results.winning_trades + results.losing_trades} closed`
                    : `${results.winning_trades}/${results.total_trades} trades`
                  }
                </div>
              </div>
            </div>

            {/* DCA Strategy Explanation */}
            {isDCA && totalInvested && (
              <div className="bg-gradient-to-r from-cyan-500/10 to-blue-500/10 border border-cyan-500/20 rounded-lg p-4 mb-6">
                <div className="flex items-start space-x-3">
                  <div className="text-cyan-400 mt-1">
                    <Award className="h-5 w-5" />
                  </div>
                  <div className="flex-1">
                    <h4 className="text-white/90 font-semibold mb-1">DCA Strategy Metrics</h4>
                    <p className="text-sm text-white/70">
                      This Dollar Cost Averaging strategy deployed <span className="font-semibold text-cyan-400">{formatCurrency(totalInvested)}</span> over time.
                      The return percentage ({formatPercentage(results.total_return_percentage)}) is calculated based on your total invested amount,
                      not the initial balance. This accurately reflects your strategy's performance.
                    </p>
                  </div>
                </div>
              </div>
            )}

            {/* Simple Equity Curve Visualization */}
            <div className="bg-black/30 rounded-lg p-4 mb-6">
              <h4 className="text-white/80 font-medium mb-4 flex items-center space-x-2">
                <TrendingUp className="h-4 w-4" />
                <span>Portfolio Growth</span>
              </h4>
              <div className="relative h-32 bg-black/20 rounded overflow-hidden">
                <svg
                  className="w-full h-full"
                  viewBox="0 0 100 100"
                  preserveAspectRatio="none"
                  style={{ display: 'block' }}
                >
                  {/* Grid lines for reference */}
                  <line x1="0" y1="25" x2="100" y2="25" stroke="rgba(255,255,255,0.05)" strokeWidth="0.2" />
                  <line x1="0" y1="50" x2="100" y2="50" stroke="rgba(255,255,255,0.1)" strokeWidth="0.3" strokeDasharray="1,1" />
                  <line x1="0" y1="75" x2="100" y2="75" stroke="rgba(255,255,255,0.05)" strokeWidth="0.2" />

                  {/* Fill area under curve */}
                  <polygon
                    points={`0,100 ${equityCurve.map(point => `${point.x},${100 - point.normalizedY}`).join(' ')} 100,100`}
                    fill={isProfit ? 'rgba(16, 185, 129, 0.1)' : 'rgba(239, 68, 68, 0.1)'}
                  />

                  {/* Main curve */}
                  <polyline
                    points={equityCurve.map(point => `${point.x},${100 - point.normalizedY}`).join(' ')}
                    fill="none"
                    stroke={isProfit ? '#10b981' : '#ef4444'}
                    strokeWidth="0.8"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    className="drop-shadow-lg"
                  />
                </svg>
                <div className="absolute bottom-2 left-2 text-xs text-white/50">
                  {isDCA && totalInvested ? `Invested: ${formatCurrency(totalInvested)}` : `Start: ${formatCurrency(results.initial_balance)}`}
                </div>
                <div className="absolute bottom-2 right-2 text-xs text-white/50">
                  End: {formatCurrency(results.final_balance)}
                </div>
              </div>
            </div>

            {/* Action Buttons */}
            <div className="flex flex-wrap gap-3">
              <Button 
                variant="outline" 
                size="sm"
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                <Download className="h-4 w-4 mr-2" />
                Export Results
              </Button>
              <Button 
                variant="outline" 
                size="sm"
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                <Share2 className="h-4 w-4 mr-2" />
                Share
              </Button>
            </div>
          </CardContent>
        </div>
      </Card>

      {/* Detailed Analysis Tabs */}
      <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
        <CardContent className="p-0">
          <Tabs value={selectedTab} onValueChange={setSelectedTab} className="w-full">
            <TabsList className="grid w-full grid-cols-5 bg-transparent border-b border-white/10">
              <TabsTrigger value="overview" className="data-[state=active]:bg-white/10">
                Overview
              </TabsTrigger>
              <TabsTrigger value="trades" className="data-[state=active]:bg-white/10">
                Trades
              </TabsTrigger>
              <TabsTrigger value="open" className="data-[state=active]:bg-white/10">
                Open Trades
              </TabsTrigger>
              <TabsTrigger value="metrics" className="data-[state=active]:bg-white/10">
                Metrics
              </TabsTrigger>
              <TabsTrigger value="analysis" className="data-[state=active]:bg-white/10">
                Analysis
              </TabsTrigger>
            </TabsList>
            
            <TabsContent value="overview" className="p-6 space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* Returns Analysis */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90">Returns Analysis</h3>
                  <div className="space-y-3">
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Total Return</span>
                      <span className={cn("font-medium", returnColor)}>
                        {formatPercentage(results.total_return_percentage)}
                      </span>
                    </div>
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Annualized Return</span>
                      <span className="text-white/90 font-medium">
                        {(parseFloat(results.total_return_percentage) * 1).toFixed(2)}%
                      </span>
                    </div>
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Volatility</span>
                      <span className="text-white/90 font-medium">
                        N/A
                      </span>
                    </div>
                  </div>
                </div>

                {/* Risk Metrics */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90">Risk Metrics</h3>
                  <div className="space-y-3">
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Max Drawdown</span>
                      <span className="text-red-400 font-medium">
                        {formatPercentage(results.max_drawdown_percentage)}
                      </span>
                    </div>
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Sharpe Ratio</span>
                      <span className="text-white/90 font-medium">
                        {results.sharpe_ratio ? parseFloat(results.sharpe_ratio).toFixed(2) : 'N/A'}
                      </span>
                    </div>
                    <div className="flex justify-between items-center p-3 bg-white/5 rounded-lg">
                      <span className="text-white/70">Sortino Ratio</span>
                      <span className="text-white/90 font-medium">
                        N/A
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>
            
            <TabsContent value="trades" className="p-6 space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-white/90">Trade History</h3>
                <Badge variant="outline" className="border-white/20 text-white/80">
                  {results.total_trades} trades
                </Badge>
              </div>
              
              <div className="space-y-2 max-h-96 overflow-y-auto">
                {results.trades_data && Array.isArray(results.trades_data) ? results.trades_data.slice(0, 20).map((trade, index) => (
                  <div key={index} className="p-3 bg-white/5 rounded-lg flex items-center justify-between">
                    <div className="flex items-center space-x-3">
                      <div className={cn(
                        "w-2 h-2 rounded-full",
                        trade.pnl && parseFloat(trade.pnl) >= 0 ? "bg-green-400" :
                        trade.trade_type === 'Buy' ? "bg-blue-400" : "bg-orange-400"
                      )} />
                      <div>
                        <div className="text-sm font-medium text-white/90">
                          {(trade.side || trade.trade_type || 'Trade').toUpperCase()} @ {formatCurrency(trade.entry_price || trade.price)}
                        </div>
                        <div className="text-xs text-white/60">
                          {new Date(trade.entry_date || trade.timestamp).toLocaleDateString()}
                        </div>
                      </div>
                    </div>
                    <div className="text-right">
                      {trade.pnl ? (
                        <div className={cn(
                          "text-sm font-medium",
                          parseFloat(trade.pnl) >= 0 ? "text-green-400" : "text-red-400"
                        )}>
                          {formatCurrency(trade.pnl)}
                        </div>
                      ) : (
                        <div className="text-sm font-medium text-white/90">
                          {formatCurrency(trade.total_value || '0')}
                        </div>
                      )}
                      <div className="text-xs text-white/60">
                        {trade.pnl_percentage ? formatPercentage(trade.pnl_percentage) : 'N/A'}
                      </div>
                    </div>
                  </div>
                )) : (
                  <div className="p-3 bg-white/5 rounded-lg text-center text-white/60">
                    No trade data available
                  </div>
                )}
              </div>
            </TabsContent>

            <TabsContent value="open" className="p-6 space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold text-white/90">Open Positions</h3>
                <Badge variant="outline" className="border-orange-500/30 text-orange-400">
                  {(results as any).open_positions?.length || 0} open
                </Badge>
              </div>

              {(results as any).open_positions && Array.isArray((results as any).open_positions) && (results as any).open_positions.length > 0 ? (
                <>
                  <div className="bg-orange-500/10 border border-orange-500/20 rounded-lg p-4 mb-4">
                    <div className="flex items-start space-x-3">
                      <div className="text-orange-400 mt-1">
                        <Target className="h-5 w-5" />
                      </div>
                      <div className="flex-1">
                        <h4 className="text-white/90 font-semibold mb-1">Unclosed Positions</h4>
                        <p className="text-sm text-white/70">
                          These are buy orders that haven't been matched with sell orders yet.
                          For grid trading, this happens when price moves away before the sell level is hit.
                        </p>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2 max-h-96 overflow-y-auto">
                    {(results as any).open_positions.map((position: any, index: number) => {
                      const currentPrice = parseFloat(results.final_balance) // Simplified - you'd want actual last price
                      const entryPrice = parseFloat(position.price)
                      const quantity = parseFloat(position.quantity)
                      const unrealizedPnl = (currentPrice - entryPrice) * quantity
                      const unrealizedPnlPct = ((currentPrice - entryPrice) / entryPrice) * 100

                      return (
                        <div key={index} className="p-3 bg-white/5 rounded-lg hover:bg-white/10 transition-colors">
                          <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-3 flex-1">
                              <div className="w-2 h-2 rounded-full bg-orange-400 animate-pulse" />
                              <div className="flex-1">
                                <div className="text-sm font-medium text-white/90">
                                  BUY @ {formatCurrency(position.price)}
                                </div>
                                <div className="text-xs text-white/60 mt-0.5">
                                  {new Date(position.timestamp).toLocaleDateString()} • Qty: {parseFloat(position.quantity).toFixed(6)}
                                </div>
                                <div className="text-xs text-white/50 mt-0.5">
                                  {position.reason || 'Grid level'}
                                </div>
                              </div>
                            </div>
                            <div className="text-right">
                              <div className="text-sm font-medium text-orange-400">
                                {formatCurrency(position.total_value)}
                              </div>
                              <div className="text-xs text-white/60">
                                Value at entry
                              </div>
                            </div>
                          </div>
                        </div>
                      )
                    })}
                  </div>
                </>
              ) : (
                <div className="p-6 bg-white/5 rounded-lg text-center">
                  <div className="text-white/60 mb-2">No open positions</div>
                  <div className="text-xs text-white/50">All buy orders have been closed with sell orders</div>
                </div>
              )}
            </TabsContent>

            <TabsContent value="metrics" className="p-6 space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {/* Trading Stats */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90">Trading Statistics</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Total Trades</span>
                      <span className="text-white/90">{results.total_trades}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Winning Trades</span>
                      <span className="text-green-400">{results.winning_trades}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Losing Trades</span>
                      <span className="text-red-400">{results.losing_trades}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Win Rate</span>
                      <span className="text-white/90">{formatPercentage(results.win_rate)}</span>
                    </div>
                  </div>
                </div>

                {/* Profit/Loss Stats */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90">P&L Statistics</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Average Win</span>
                      <span className="text-green-400">{formatCurrency(results.average_win)}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Average Loss</span>
                      <span className="text-red-400">{formatCurrency(results.average_loss)}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Profit Factor</span>
                      <span className="text-white/90">{results.profit_factor ? parseFloat(results.profit_factor).toFixed(2) : 'N/A'}</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Best Day</span>
                      <span className="text-green-400">N/A</span>
                    </div>
                  </div>
                </div>

                {/* Advanced Metrics */}
                <div className="space-y-4">
                  <h3 className="text-lg font-semibold text-white/90">Advanced Metrics</h3>
                  <div className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Calmar Ratio</span>
                      <span className="text-white/90">N/A</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Recovery Factor</span>
                      <span className="text-white/90">N/A</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Max Consecutive Losses</span>
                      <span className="text-red-400">N/A</span>
                    </div>
                    <div className="flex justify-between text-sm">
                      <span className="text-white/70">Worst Day</span>
                      <span className="text-red-400">N/A</span>
                    </div>
                  </div>
                </div>
              </div>
            </TabsContent>
            
            <TabsContent value="analysis" className="p-6 space-y-6">
              {/* Performance Analysis */}
              <div className="space-y-4">
                <h3 className="text-lg font-semibold text-white/90">Performance Analysis</h3>
                
                {parseFloat(results.total_return_percentage) > 0 ? (
                  <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-4">
                    <div className="flex items-center space-x-2 mb-2">
                      <TrendingUp className="h-5 w-5 text-green-400" />
                      <span className="font-medium text-green-400">Profitable Strategy</span>
                    </div>
                    <ul className="text-sm text-white/70 space-y-1">
                      <li>• Strategy generated positive returns over the backtest period</li>
                      <li>• Consider risk management settings and position sizing</li>
                      <li>• Review drawdown periods for potential improvements</li>
                    </ul>
                  </div>
                ) : (
                  <div className="bg-red-500/10 border border-red-500/20 rounded-lg p-4">
                    <div className="flex items-center space-x-2 mb-2">
                      <TrendingDown className="h-5 w-5 text-red-400" />
                      <span className="font-medium text-red-400">Strategy Needs Optimization</span>
                    </div>
                    <ul className="text-sm text-white/70 space-y-1">
                      <li>• Strategy lost money over the backtest period</li>
                      <li>• Consider adjusting parameters or risk management</li>
                      <li>• Test with different time periods or market conditions</li>
                    </ul>
                  </div>
                )}

                {/* Recommendations */}
                <div className="bg-blue-500/10 border border-blue-500/20 rounded-lg p-4">
                  <div className="flex items-center space-x-2 mb-2">
                    <Target className="h-5 w-5 text-blue-400" />
                    <span className="font-medium text-blue-400">Recommendations</span>
                  </div>
                  <ul className="text-sm text-white/70 space-y-1">
                    {results.sharpe_ratio && parseFloat(results.sharpe_ratio) < 1 && (
                      <li>• Low Sharpe ratio suggests high risk-adjusted returns could be improved</li>
                    )}
                    {parseFloat(results.win_rate) < 40 && (
                      <li>• Low win rate indicates strategy needs better entry/exit signals</li>
                    )}
                    {parseFloat(results.max_drawdown_percentage) > 20 && (
                      <li>• High max drawdown suggests implementing stricter risk controls</li>
                    )}
                    <li>• Test with different market conditions and time periods</li>
                    <li>• Consider paper trading before live deployment</li>
                  </ul>
                </div>
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}

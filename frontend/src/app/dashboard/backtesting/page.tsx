"use client"

import { useState, useEffect, useCallback, useMemo } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  TestTube,
  BarChart3,
  Calendar,
  RefreshCw,
  Trash2,
  Award,
  Target
} from "lucide-react"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { StrategyBacktestSelector } from "@/components/backtesting/strategy-backtest-selector"
import { BacktestResults } from "@/components/backtesting/backtest-results"
import { DCAConfig } from "@/components/strategies/config/dca-config"
import { GridTradingConfig } from "@/components/strategies/config/grid-trading-config"
import { SMAConfig } from "@/components/strategies/config/sma-crossover-config"
import {
  apiClient,
  StrategyType,
  BacktestResult,
  BacktestEngineResult,
  BacktestRequest,
  type DCAConfig as DCAConfigType,
  type GridTradingConfig as GridConfigType,
  type SMACrossoverConfig as SMAConfigType
} from "@/lib/api"
import { 
  getStrategyInfo, 
  formatCurrency, 
  formatPercentage,
  DEFAULT_CONFIGS 
} from "@/lib/strategies"
import { useAuth } from "@/contexts/auth-context"
import { cn } from "@/lib/utils"

type ViewMode = 'overview' | 'select' | 'configure' | 'running' | 'results'

export default function BacktestingPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [viewMode, setViewMode] = useState<ViewMode>('overview')

  // Helper function to map backend strategy name to frontend strategy type
  const getStrategyTypeFromName = (strategyName: string): StrategyType => {
    const nameToTypeMap: Record<string, StrategyType> = {
      'dca_v2': 'dca',
      'grid_trading_v2': 'grid_trading',
      'sma_crossover_v2': 'sma_crossover'
    }
    return nameToTypeMap[strategyName] || 'dca' // fallback to dca if not found
  }

  // Helper function to check if result is from engine or database
  const isBacktestEngineResult = (result: BacktestResult | BacktestEngineResult): result is BacktestEngineResult => {
    return 'metrics' in result && 'trades' in result && 'performance_chart' in result
  }

  // Helper function to convert BacktestEngineResult to BacktestResult format for component compatibility
  const convertEngineResultToDisplayFormat = (engineResult: BacktestEngineResult): BacktestResult & { equity_curve: unknown; trades_data: unknown; total_invested?: string } => {
    const symbol = engineResult.config?.symbol || 'BTC'
    const strategyName = engineResult.config?.strategy_name || 'dca_v2'
    const interval = engineResult.config?.interval || '1d'
    // Backend uses start_time/end_time, not start_date/end_date
    const config = engineResult.config as { start_time?: string; end_time?: string; start_date?: string; end_date?: string; initial_balance?: number; [key: string]: unknown }
    const startDate = config?.start_time || config?.start_date || new Date().toISOString()
    const endDate = config?.end_time || config?.end_date || new Date().toISOString()
    const initialBalance = engineResult.config?.initial_balance?.toString() || '10000'

    return {
      id: engineResult.backtest_id,
      name: `${strategyName} on ${symbol}`,
      description: `Backtest of ${strategyName} strategy`,
      strategy_name: strategyName,
      symbol: symbol,
      interval: interval,
      start_date: startDate,
      end_date: endDate,
      initial_balance: initialBalance,
      final_balance: engineResult.metrics.final_portfolio_value,
      total_return: engineResult.metrics.total_return,
      total_return_percentage: engineResult.metrics.total_return_percentage,
      max_drawdown: engineResult.metrics.max_drawdown,
      max_drawdown_percentage: (parseFloat(engineResult.metrics.max_drawdown) / parseFloat(initialBalance) * 100).toString(),
      sharpe_ratio: engineResult.metrics.sharpe_ratio,
      total_trades: engineResult.metrics.total_trades,
      winning_trades: engineResult.metrics.winning_trades,
      losing_trades: engineResult.metrics.losing_trades,
      win_rate: engineResult.metrics.win_rate,
      profit_factor: engineResult.metrics.profit_factor,
      largest_win: '0', // Not available in engine result
      largest_loss: '0', // Not available in engine result
      average_win: engineResult.metrics.average_win,
      average_loss: engineResult.metrics.average_loss,
      status: 'completed',
      error_message: undefined,
      execution_time_ms: engineResult.execution_time_ms,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      // Additional fields for compatibility
      equity_curve: engineResult.performance_chart,
      trades_data: engineResult.trades,
      // DCA-specific field
      total_invested: engineResult.metrics.total_invested
    }
  }
  const [selectedStrategyType, setSelectedStrategyType] = useState<StrategyType>()
  const [customDCAConfig, setCustomDCAConfig] = useState<DCAConfigType | null>(null)
  const [customGridConfig, setCustomGridConfig] = useState<GridConfigType | null>(null)
  const [customSMAConfig, setCustomSMAConfig] = useState<SMAConfigType | null>(null)
  const [backtestHistory, setBacktestHistory] = useState<BacktestResult[]>([])
  const [currentBacktest, setCurrentBacktest] = useState<BacktestResult | BacktestEngineResult | null>(null)
  const [loading, setLoading] = useState(false)

  // Memoized data and callbacks for strategy configs (must be outside conditionals)
  const dcaInitialData = useMemo(() => ({
    name: selectedStrategyType === 'dca' ? `${getStrategyInfo(selectedStrategyType).name} Backtest` : '',
    asset_symbol: 'BTC',
    config: customDCAConfig || (selectedStrategyType === 'dca' ? DEFAULT_CONFIGS[selectedStrategyType] as DCAConfigType : {} as DCAConfigType)
  }), [selectedStrategyType, customDCAConfig])

  const gridInitialData = useMemo(() => ({
    name: selectedStrategyType === 'grid_trading' ? `${getStrategyInfo(selectedStrategyType).name} Backtest` : '',
    asset_symbol: 'BTC',
    config: customGridConfig || (selectedStrategyType === 'grid_trading' ? DEFAULT_CONFIGS[selectedStrategyType] as GridConfigType : {} as GridConfigType)
  }), [selectedStrategyType, customGridConfig])

  const smaInitialData = useMemo(() => ({
    name: selectedStrategyType === 'sma_crossover' ? `${getStrategyInfo(selectedStrategyType).name} Backtest` : '',
    asset_symbol: 'BTC',
    config: customSMAConfig || (selectedStrategyType === 'sma_crossover' ? DEFAULT_CONFIGS[selectedStrategyType] as SMAConfigType : {} as SMAConfigType)
  }), [selectedStrategyType, customSMAConfig])

  const handleConfigSubmit = useCallback(async () => {
    // This is for strategy creation, not backtesting
    // We'll handle this in the onBacktest callback
  }, [])

  const handleConfigCancel = useCallback(() => setViewMode('select'), [])

  const loadBacktestHistory = useCallback(async () => {
    if (!isAuthenticated) return

    setLoading(true)
    try {
      const response = await apiClient.getUserBacktests({ page: 1, limit: 50 })
      setBacktestHistory(response.results || [])
    } catch (error: unknown) {
      console.error('Failed to load backtest history:', error)
      // Check if it's a 500 error (backend not implemented)
      if ((error as { status?: number }).status === 500 || (error as { message?: string }).message?.includes('500')) {
        console.warn('Backtesting endpoints not yet implemented on backend')
      }
      setBacktestHistory([])
    } finally {
      setLoading(false)
    }
  }, [isAuthenticated])

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadBacktestHistory()

      // Check if there's a pending strategy from the unified strategies page
      const pendingStrategy = localStorage.getItem('pendingBacktestStrategy')
      if (pendingStrategy) {
        try {
          const strategyData = JSON.parse(pendingStrategy)
          setSelectedStrategyType(strategyData.type)

          // Set the appropriate custom config based on strategy type
          switch (strategyData.type) {
            case 'dca':
              setCustomDCAConfig(strategyData.config)
              break
            case 'grid_trading':
              setCustomGridConfig(strategyData.config)
              break
            case 'sma_crossover':
              setCustomSMAConfig(strategyData.config)
              break
          }

          setViewMode('configure')
          localStorage.removeItem('pendingBacktestStrategy')
        } catch (error) {
          console.error('Error parsing pending strategy:', error)
          localStorage.removeItem('pendingBacktestStrategy')
        }
      }
    }
  }, [isAuthenticated, authLoading, loadBacktestHistory])

  const handleStrategySelect = (type: StrategyType) => {
    setSelectedStrategyType(type)
    setViewMode('configure')
  }

  const handleConfigureStrategy = (type: StrategyType) => {
    setSelectedStrategyType(type)
    // Reset all custom configs
    setCustomDCAConfig(null)
    setCustomGridConfig(null)
    setCustomSMAConfig(null)
    setViewMode('configure')
  }

  const handleDCABacktest = async (
    config: DCAConfigType,
    name: string,
    assetSymbol: string,
    backtestParams: {
      start_date: string
      end_date: string
      interval: string
      initial_capital: number
    }
  ) => {
    setCustomDCAConfig(config)
    setViewMode('running')

    try {
      const request: BacktestRequest = {
        strategy_type: 'dca',
        asset_symbol: assetSymbol || 'BTC',
        start_date: backtestParams.start_date,
        end_date: backtestParams.end_date,
        initial_capital: backtestParams.initial_capital,
        config: config,
        interval: backtestParams.interval
      }
      await handleRunBacktest(request)
    } catch (error) {
      console.error('DCA backtest failed:', error)
      setViewMode('configure')
    }
  }

  const handleGridTradingBacktest = async (
    config: GridConfigType,
    name: string,
    assetSymbol: string,
    backtestParams: {
      start_date: string
      end_date: string
      interval: string
      initial_capital: number
    }
  ) => {
    setCustomGridConfig(config)
    setViewMode('running')

    try {
      const request: BacktestRequest = {
        strategy_type: 'grid_trading',
        asset_symbol: assetSymbol || 'BTC',
        start_date: backtestParams.start_date,
        end_date: backtestParams.end_date,
        initial_capital: backtestParams.initial_capital,
        config: config,
        interval: backtestParams.interval
      }
      await handleRunBacktest(request)
    } catch (error) {
      console.error('Grid trading backtest failed:', error)
      setViewMode('configure')
    }
  }

  const handleSMABacktest = async (
    config: SMAConfigType,
    name: string,
    assetSymbol: string,
    backtestParams: {
      start_date: string
      end_date: string
      interval: string
      initial_capital: number
    }
  ) => {
    setCustomSMAConfig(config)
    setViewMode('running')

    try {
      const request: BacktestRequest = {
        strategy_type: 'sma_crossover',
        asset_symbol: assetSymbol || 'BTC',
        start_date: backtestParams.start_date,
        end_date: backtestParams.end_date,
        initial_capital: backtestParams.initial_capital,
        config: config,
        interval: backtestParams.interval
      }
      await handleRunBacktest(request)
    } catch (error) {
      console.error('SMA crossover backtest failed:', error)
      setViewMode('configure')
    }
  }


  const handleRunBacktest = async (request: BacktestRequest) => {
    setViewMode('running')
    try {
      const result = await apiClient.runBacktest(request)
      setCurrentBacktest(result)
      setViewMode('results')
      await loadBacktestHistory() // Refresh history
    } catch (error) {
      console.error('Backtest failed:', error)
      setViewMode('configure')
    }
  }

  const handleDeleteBacktest = async (backtestId: string) => {
    if (!confirm('Are you sure you want to delete this backtest result?')) return
    
    try {
      await apiClient.deleteBacktest(backtestId)
      await loadBacktestHistory()
    } catch (error) {
      console.error('Failed to delete backtest:', error)
    }
  }


  const getPerformanceRating = (returnPercentage: string, sharpeRatio: string) => {
    const returns = parseFloat(returnPercentage)
    const sharpe = parseFloat(sharpeRatio)
    
    if (returns > 20 && sharpe > 1.5) return { rating: 'Excellent', color: 'text-green-400' }
    if (returns > 10 && sharpe > 1.0) return { rating: 'Good', color: 'text-blue-400' }
    if (returns > 0 && sharpe > 0.5) return { rating: 'Fair', color: 'text-yellow-400' }
    return { rating: 'Poor', color: 'text-red-400' }
  }

  if (authLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
        </div>
      </DashboardLayout>
    )
  }

  if (!isAuthenticated) {
    return (
      <DashboardLayout>
        <div className="text-center py-12">
          <h2 className="text-2xl font-bold text-white/90 mb-4">Authentication Required</h2>
          <p className="text-white/60">Please log in to access the backtesting lab.</p>
        </div>
      </DashboardLayout>
    )
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center space-y-4 sm:space-y-0">
          <div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-cyan-400 to-blue-400 bg-clip-text text-transparent">
              Backtesting Lab
            </h1>
            <p className="text-white/60 mt-1">
              Test and optimize your trading strategies with historical data
            </p>
          </div>
          
          <div className="flex items-center space-x-3">
            <Button
              onClick={loadBacktestHistory}
              variant="outline"
              size="sm"
              disabled={loading}
              className="border-white/20 text-white/80 hover:bg-white/10"
            >
              <RefreshCw className={cn("h-4 w-4 mr-2", loading && "animate-spin")} />
              Refresh
            </Button>
            
            <Button
              onClick={() => setViewMode('select')}
              className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500"
            >
              <TestTube className="h-4 w-4 mr-2" />
              New Backtest
            </Button>
          </div>
        </div>

        {viewMode === 'overview' && (
          <>
            {/* Quick Stats */}
            <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-8 max-w-6xl mx-auto">
              <Card className="bg-gradient-to-br from-cyan-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <TestTube className="h-8 w-8 text-cyan-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">{backtestHistory.length}</p>
                      <p className="text-sm text-white/60">Total Backtests</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-gradient-to-br from-green-500/20 to-emerald-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <Award className="h-8 w-8 text-green-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">
                        {backtestHistory.filter(b => parseFloat(b.total_return_percentage) > 0).length}
                      </p>
                      <p className="text-sm text-white/60">Profitable Tests</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-gradient-to-br from-yellow-500/20 to-orange-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <BarChart3 className="h-8 w-8 text-yellow-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">
                        {backtestHistory.length > 0 
                          ? (backtestHistory.reduce((sum, b) => sum + parseFloat(b.total_return_percentage), 0) / backtestHistory.length).toFixed(1)
                          : '0'
                        }%
                      </p>
                      <p className="text-sm text-white/60">Avg Return</p>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-gradient-to-br from-purple-500/20 to-pink-500/20 backdrop-blur-xl border border-white/10">
                <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
                <CardContent className="relative z-10 p-6">
                  <div className="flex items-center space-x-2">
                    <Target className="h-8 w-8 text-purple-400" />
                    <div>
                      <p className="text-2xl font-bold text-white">
                        {backtestHistory.length > 0 
                          ? Math.max(...backtestHistory.map(b => parseFloat(b.total_return_percentage))).toFixed(1)
                          : '0'
                        }%
                      </p>
                      <p className="text-sm text-white/60">Best Return</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Recent Backtests */}
            <Card className="bg-white/5 backdrop-blur-xl border border-white/10 max-w-7xl mx-auto">
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-xl font-bold text-white/90">
                      Backtest History
                    </CardTitle>
                    <CardDescription className="text-white/60">
                      Your recent strategy testing results
                    </CardDescription>
                  </div>
                  <Button
                    onClick={() => setViewMode('select')}
                    size="sm"
                    className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500"
                  >
                    <TestTube className="h-4 w-4 mr-2" />
                    New Test
                  </Button>
                </div>
              </CardHeader>

              <CardContent>
                {loading ? (
                  <div className="flex items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
                  </div>
                ) : backtestHistory.length === 0 ? (
                  <div className="text-center py-12">
                    <TestTube className="h-16 w-16 text-white/40 mx-auto mb-4" />
                    <h3 className="text-xl font-semibold text-white/90 mb-2">No Backtests Yet</h3>
                    <p className="text-white/60 mb-6">
                      Start testing strategies with historical data to see how they would have performed
                    </p>
                    <Button
                      onClick={() => setViewMode('select')}
                      className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500"
                    >
                      <TestTube className="h-4 w-4 mr-2" />
                      Run Your First Backtest
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {backtestHistory.slice(0, 10).map((backtest) => {
                      const strategyType = getStrategyTypeFromName(backtest.strategy_name)
                      const strategyInfo = getStrategyInfo(strategyType)
                      const performance = getPerformanceRating(backtest.total_return_percentage, backtest.sharpe_ratio || '0')
                      const isProfit = parseFloat(backtest.total_return_percentage) >= 0
                      
                      return (
                        <Card
                          key={backtest.id}
                          className={cn(
                            "bg-gradient-to-r backdrop-blur-sm border border-white/10 cursor-pointer transition-all hover:scale-[1.01]",
                            "min-h-[120px]", // Increased height for better readability
                            strategyInfo.color
                          )}
                          onClick={() => {
                            setCurrentBacktest(backtest)
                            setViewMode('results')
                          }}
                        >
                          <CardContent className="p-4">
                            <div className="flex items-center justify-between">
                              <div className="flex items-center space-x-4">
                                <div className="text-2xl">{strategyInfo.icon}</div>
                                <div>
                                  <h4 className="font-semibold text-white/90">
                                    {strategyInfo.name} • {backtest.symbol}
                                  </h4>
                                  <div className="flex items-center space-x-3 text-sm text-white/60">
                                    <span>{new Date(backtest.start_date).toLocaleDateString()} - {new Date(backtest.end_date).toLocaleDateString()}</span>
                                    <Badge className={cn("border", performance.color)}>
                                      {performance.rating}
                                    </Badge>
                                  </div>
                                </div>
                              </div>
                              
                              <div className="text-right space-y-1">
                                <div className={cn(
                                  "text-lg font-bold",
                                  isProfit ? "text-green-400" : "text-red-400"
                                )}>
                                  {formatPercentage(backtest.total_return_percentage)}
                                </div>
                                <div className="text-sm text-white/60">
                                  {formatCurrency(backtest.total_return)} • {backtest.total_trades} trades
                                </div>
                                <div className="flex items-center space-x-2">
                                  <Button
                                    size="sm"
                                    variant="outline"
                                    onClick={(e) => {
                                      e.stopPropagation()
                                      handleDeleteBacktest(backtest.id)
                                    }}
                                    className="border-red-500/30 text-red-400 hover:bg-red-500/10 h-6 px-2"
                                  >
                                    <Trash2 className="h-3 w-3" />
                                  </Button>
                                </div>
                              </div>
                            </div>
                          </CardContent>
                        </Card>
                      )
                    })}
                  </div>
                )}
              </CardContent>
            </Card>
          </>
        )}

        {viewMode === 'select' && (
          <div className="space-y-8">
            <StrategyBacktestSelector
              selectedType={selectedStrategyType}
              onStrategySelect={handleStrategySelect}
              onConfigureStrategy={handleConfigureStrategy}
              className="max-w-7xl mx-auto"
            />
          </div>
        )}

        {viewMode === 'configure' && selectedStrategyType && (
          <div className="space-y-6">
            <div className="flex items-center space-x-4">
              <Button
                onClick={() => setViewMode('select')}
                variant="outline"
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                ← Back to Strategy Selection
              </Button>
              <h2 className="text-xl font-bold text-white/90">
                Configure {getStrategyInfo(selectedStrategyType).name} Strategy
              </h2>
            </div>

            <div className="max-w-6xl mx-auto">
              {selectedStrategyType === 'dca' && (
                <DCAConfig
                  initialData={dcaInitialData}
                  onSubmit={handleConfigSubmit}
                  onCancel={handleConfigCancel}
                  onBacktest={handleDCABacktest}
                  className="w-full"
                />
              )}

              {selectedStrategyType === 'grid_trading' && (
                <GridTradingConfig
                  initialData={gridInitialData}
                  onSubmit={handleConfigSubmit}
                  onCancel={handleConfigCancel}
                  onBacktest={handleGridTradingBacktest}
                  className="w-full"
                />
              )}

              {selectedStrategyType === 'sma_crossover' && (
                <SMAConfig
                  initialData={smaInitialData}
                  onSubmit={handleConfigSubmit}
                  onCancel={handleConfigCancel}
                  onBacktest={handleSMABacktest}
                  className="w-full"
                />
              )}
            </div>
          </div>
        )}

        {viewMode === 'running' && (
          <Card className="bg-gradient-to-br from-cyan-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <div className="relative z-10">
              <CardContent className="p-12 text-center">
                <div className="animate-spin rounded-full h-16 w-16 border-b-4 border-cyan-400 mx-auto mb-6"></div>
                <h3 className="text-2xl font-bold text-white/90 mb-2">
                  Running Backtest...
                </h3>
                <p className="text-white/60 mb-4">
                  Analyzing historical data and calculating performance metrics
                </p>
                <div className="text-sm text-white/50">
                  This may take a few moments depending on the date range and strategy complexity
                </div>
              </CardContent>
            </div>
          </Card>
        )}

        {viewMode === 'results' && currentBacktest && (
          <div className="space-y-6 max-w-7xl mx-auto">
            <div className="flex items-center space-x-4">
              <Button
                onClick={() => setViewMode('overview')}
                variant="outline"
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                ← Back to Overview
              </Button>
              <h2 className="text-xl font-bold text-white/90">
                Backtest Results: {getStrategyInfo(getStrategyTypeFromName(
                  isBacktestEngineResult(currentBacktest)
                    ? currentBacktest.config?.strategy_name || 'dca_v2'
                    : currentBacktest.strategy_name
                )).name}
              </h2>
            </div>

            <BacktestResults
              results={
                isBacktestEngineResult(currentBacktest)
                  ? convertEngineResultToDisplayFormat(currentBacktest)
                  : currentBacktest
              }
              strategyInfo={getStrategyInfo(getStrategyTypeFromName(
                isBacktestEngineResult(currentBacktest)
                  ? currentBacktest.config?.strategy_name || 'dca_v2'
                  : currentBacktest.strategy_name
              ))}
              className="w-full"
            />
          </div>
        )}

        {/* Strategy Quick Start */}
        {viewMode === 'overview' && backtestHistory.length === 0 && (
          <Card className="bg-gradient-to-br from-cyan-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10 max-w-6xl mx-auto">
            <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
            <div className="relative z-10">
              <CardHeader className="text-center space-y-4">
                <div className="text-4xl">🧪</div>
                <CardTitle className="text-2xl font-bold text-white/90">
                  Welcome to the Backtesting Lab
                </CardTitle>
                <CardDescription className="text-white/60 max-w-2xl mx-auto">
                  Test any trading strategy with historical market data before risking real money. 
                  Our backtesting engine provides comprehensive performance analysis with detailed metrics.
                </CardDescription>
              </CardHeader>

              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 max-w-4xl mx-auto">
                  <div className="text-center p-6 bg-white/5 rounded-lg border border-white/10">
                    <BarChart3 className="h-8 w-8 text-blue-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Performance Metrics</h4>
                    <p className="text-sm text-white/60">
                      Sharpe ratio, max drawdown, win rate, and more
                    </p>
                  </div>
                  
                  <div className="text-center p-6 bg-white/5 rounded-lg border border-white/10">
                    <Calendar className="h-8 w-8 text-cyan-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Historical Data</h4>
                    <p className="text-sm text-white/60">
                      Test with up to 5 years of market data
                    </p>
                  </div>
                  
                  <div className="text-center p-6 bg-white/5 rounded-lg border border-white/10">
                    <Target className="h-8 w-8 text-purple-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Risk Analysis</h4>
                    <p className="text-sm text-white/60">
                      Understand potential risks before going live
                    </p>
                  </div>
                </div>

                <div className="text-center">
                  <Button
                    onClick={() => setViewMode('select')}
                    size="lg"
                    className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500 text-white"
                  >
                    <TestTube className="h-5 w-5 mr-2" />
                    Start Your First Backtest
                  </Button>
                </div>
              </CardContent>
            </div>
          </Card>
        )}
      </div>
    </DashboardLayout>
  )
}

"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { 
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { 
  TestTube, 
  Play, 
  BarChart3, 
  TrendingUp, 
  TrendingDown,
  Calendar,
  DollarSign,
  RefreshCw,
  Download,
  Trash2,
  Award,
  AlertTriangle,
  Target,
  Activity,
  PlusCircle
} from "lucide-react"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { StrategyTypeSelector } from "@/components/strategies/strategy-type-selector"
import { BacktestingInterface } from "@/components/backtesting/backtesting-interface"
import { BacktestResults } from "@/components/backtesting/backtest-results"
import { 
  apiClient, 
  StrategyType, 
  BacktestResult, 
  BacktestRequest,
  StrategyConfig
} from "@/lib/api"
import { 
  getStrategyInfo, 
  formatCurrency, 
  formatPercentage,
  DEFAULT_CONFIGS 
} from "@/lib/strategies"
import { useAuth } from "@/contexts/auth-context"
import { cn } from "@/lib/utils"

type ViewMode = 'overview' | 'create' | 'running' | 'results'

export default function BacktestingPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [viewMode, setViewMode] = useState<ViewMode>('overview')
  const [selectedStrategyType, setSelectedStrategyType] = useState<StrategyType>()
  const [backtestHistory, setBacktestHistory] = useState<BacktestResult[]>([])
  const [currentBacktest, setCurrentBacktest] = useState<BacktestResult | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadBacktestHistory()
    }
  }, [isAuthenticated, authLoading])

  const loadBacktestHistory = async () => {
    if (!isAuthenticated) return
    
    setLoading(true)
    try {
      const history = await apiClient.getUserBacktests()
      setBacktestHistory(history || [])
    } catch (error) {
      console.error('Failed to load backtest history:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleStartBacktest = (type: StrategyType) => {
    setSelectedStrategyType(type)
    setViewMode('create')
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
      setViewMode('create')
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

  const getPerformanceColor = (returnPercentage: string) => {
    const value = parseFloat(returnPercentage)
    if (value >= 20) return 'text-green-400'
    if (value >= 10) return 'text-blue-400'
    if (value >= 0) return 'text-yellow-400'
    return 'text-red-400'
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
              onClick={() => setViewMode('create')}
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
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
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
            <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
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
                    onClick={() => setViewMode('create')}
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
                      onClick={() => setViewMode('create')}
                      className="bg-gradient-to-r from-cyan-600 to-blue-600 hover:from-cyan-500 hover:to-blue-500"
                    >
                      <TestTube className="h-4 w-4 mr-2" />
                      Run Your First Backtest
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {backtestHistory.slice(0, 10).map((backtest) => {
                      const strategyInfo = getStrategyInfo(backtest.strategy_type)
                      const performance = getPerformanceRating(backtest.total_return_percentage, backtest.sharpe_ratio)
                      const isProfit = parseFloat(backtest.total_return_percentage) >= 0
                      
                      return (
                        <Card 
                          key={backtest.id}
                          className={cn(
                            "bg-gradient-to-r backdrop-blur-sm border border-white/10 cursor-pointer transition-all hover:scale-[1.01]",
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
                                    {strategyInfo.name} • {backtest.asset_symbol}
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

        {viewMode === 'create' && (
          <div className="space-y-6">
            <Card className="bg-white/5 backdrop-blur-xl border border-white/10">
              <CardHeader className="text-center">
                <CardTitle className="text-2xl font-bold text-white/90">
                  Choose Strategy to Backtest
                </CardTitle>
                <CardDescription className="text-white/60">
                  Select a strategy type and we'll guide you through the configuration
                </CardDescription>
              </CardHeader>
              <CardContent>
                <StrategyTypeSelector
                  onTypeSelect={handleStartBacktest}
                  className="max-w-4xl mx-auto"
                />
              </CardContent>
            </Card>
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
          <div className="space-y-6">
            <div className="flex items-center space-x-4">
              <Button
                onClick={() => setViewMode('overview')}
                variant="outline"
                className="border-white/20 text-white/80 hover:bg-white/10"
              >
                ← Back to Overview
              </Button>
              <h2 className="text-xl font-bold text-white/90">
                Backtest Results: {getStrategyInfo(currentBacktest.strategy_type).name}
              </h2>
            </div>
            
            <BacktestResults 
              results={currentBacktest}
              strategyInfo={getStrategyInfo(currentBacktest.strategy_type)}
            />
          </div>
        )}

        {/* Strategy Quick Start */}
        {viewMode === 'overview' && backtestHistory.length === 0 && (
          <Card className="bg-gradient-to-br from-cyan-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10">
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
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="text-center p-4 bg-white/5 rounded-lg">
                    <BarChart3 className="h-8 w-8 text-blue-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Performance Metrics</h4>
                    <p className="text-sm text-white/60">
                      Sharpe ratio, max drawdown, win rate, and more
                    </p>
                  </div>
                  
                  <div className="text-center p-4 bg-white/5 rounded-lg">
                    <Calendar className="h-8 w-8 text-cyan-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Historical Data</h4>
                    <p className="text-sm text-white/60">
                      Test with up to 5 years of market data
                    </p>
                  </div>
                  
                  <div className="text-center p-4 bg-white/5 rounded-lg">
                    <Target className="h-8 w-8 text-purple-400 mx-auto mb-2" />
                    <h4 className="font-semibold text-white/90">Risk Analysis</h4>
                    <p className="text-sm text-white/60">
                      Understand potential risks before going live
                    </p>
                  </div>
                </div>

                <div className="text-center">
                  <Button
                    onClick={() => setViewMode('create')}
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

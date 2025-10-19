"use client"

import { useState, useEffect } from "react"
import {
  TrendingUp,
  DollarSign,
  Activity,
  PlusCircle,
  Play,
  Pause,
  AlertTriangle,
  ArrowRight,
  Settings,
  Shield,
  Target
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { ConnectExchangePrompt } from "@/components/dashboard/connect-exchange-prompt"
import { MultiStrategyOverview } from "@/components/dashboard/multi-strategy-overview"
import { StrategyTypeStats } from "@/components/dashboard/strategy-type-stats"
import { QuickActionsPanel } from "@/components/dashboard/quick-actions-panel"
import { DxyIndicator } from "@/components/dashboard/dxy-indicator"
import { BtcDominanceIndicator, M2Indicator, BtcPriceIndicator, FearGreedIndicator } from "@/components/dashboard/market-indicators"
import { apiClient, DCAStrategy, ExchangeConnection, Strategy, StrategyType } from "@/lib/api"
import { useAuth } from "@/contexts/auth-context"
import Link from "next/link"

export default function Dashboard() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [strategies, setStrategies] = useState<DCAStrategy[]>([])
  const [allStrategies, setAllStrategies] = useState<{
    dca: Strategy[]
    gridTrading: Strategy[]
    smaCrossover: Strategy[]
  }>({
    dca: [],
    gridTrading: [],
    smaCrossover: []
  })
  const [connections, setConnections] = useState<ExchangeConnection[]>([])
  const [liveBalances, setLiveBalances] = useState<any>(null)
  const [loading, setLoading] = useState(false)
  const [liveBalanceLoading, setLiveBalanceLoading] = useState(false)
  const [summary, setSummary] = useState({
    total_allocation: "0",
    total_invested: "0",
    total_profit_loss: "0",
    active_strategies: 0,
    total_strategies: 0,
    strategy_breakdown: {
      dca: 0,
      grid_trading: 0,
      sma_crossover: 0
    }
  })

  // SECURE: Only load dashboard data when authenticated
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadDashboardData()
    } else if (!authLoading && !isAuthenticated) {
      // User is not authenticated, clear any existing data
      setStrategies([])
      setConnections([])
      setLiveBalances(null)
      setLoading(false)
    }
  }, [isAuthenticated, authLoading])

  const loadDashboardData = async () => {
    // SECURE: Double-check authentication before API calls
    if (!isAuthenticated) {
      console.warn('Attempted to load dashboard data without authentication')
      return
    }

    try {
      setLoading(true)

      // STEP 1: Get lightweight summary of user's strategy types
      const [strategySummary, connectionsRes] = await Promise.all([
        apiClient.getUserStrategySummary().catch(() => ({
          authenticated: false,
          strategy_types: [],
          total_strategies: 0,
          total_active: 0
        })),
        apiClient.getExchangeConnections().catch(() => ({ connections: [], message: '' }))
      ])

      setConnections(connectionsRes.connections || [])

      // If user has no strategies, return early with empty data
      if (!strategySummary.authenticated || strategySummary.total_strategies === 0) {
        setAllStrategies({
          dca: [],
          gridTrading: [],
          smaCrossover: []
        })
        setStrategies([])
        setSummary({
          total_allocation: "0",
          total_invested: "0",
          total_profit_loss: "0",
          active_strategies: 0,
          total_strategies: 0,
          strategy_breakdown: {
            dca: 0,
            grid_trading: 0,
            sma_crossover: 0
          }
        })
        return
      }

      // STEP 2: Only load strategy data for types the user actually has
      const activeStrategyTypes = strategySummary.strategy_types.map(st => st.strategy_type)
      console.log('Loading strategies for types:', activeStrategyTypes)

      const strategiesData = await apiClient.getStrategiesByTypes(activeStrategyTypes).catch(() => ({}))

      // Set individual strategy collections
      setAllStrategies({
        dca: strategiesData.dca?.strategies || [],
        gridTrading: strategiesData.gridTrading?.strategies || [],
        smaCrossover: strategiesData.smaCrossover?.strategies || []
      })

      // Keep DCA strategies for backward compatibility
      setStrategies(strategiesData.dca?.strategies || [])

      // Calculate combined summary statistics from loaded data
      const loadedResults = [
        strategiesData.dca,
        strategiesData.gridTrading,
        strategiesData.smaCrossover
      ].filter(Boolean) // Remove undefined entries

      const combinedSummary = {
        total_allocation: loadedResults.reduce((sum, res) => sum + parseFloat(res?.total_allocation || '0'), 0).toString(),
        total_invested: loadedResults.reduce((sum, res) => sum + parseFloat(res?.total_invested || '0'), 0).toString(),
        total_profit_loss: loadedResults.reduce((sum, res) => sum + parseFloat(res?.total_profit_loss || '0'), 0).toString(),
        active_strategies: loadedResults.reduce((sum, res) => sum + (res?.active_strategies || 0), 0),
        total_strategies: loadedResults.reduce((sum, res) => sum + (res?.strategies?.length || 0), 0),
        strategy_breakdown: {
          dca: strategiesData.dca?.strategies?.length || 0,
          grid_trading: strategiesData.gridTrading?.strategies?.length || 0,
          sma_crossover: strategiesData.smaCrossover?.strategies?.length || 0
        }
      }

      setSummary(combinedSummary)
    } catch (error) {
      console.error('Failed to load dashboard data:', error)
    } finally {
      setLoading(false)
    }
  }

  const loadLiveBalances = async () => {
    if (!isAuthenticated || liveBalanceLoading || connections.length === 0) return

    try {
      setLiveBalanceLoading(true)
      const password = prompt("Enter your password to load live balances:")
      if (!password) {
        setLiveBalanceLoading(false)
        return
      }

      // Fetch live balances for all connections
      const liveBalancesResponse = await apiClient.getAllLiveUserBalances(password)
      setLiveBalances(liveBalancesResponse)
    } catch (error) {
      console.error('Failed to load live balances:', error)
      alert('Failed to load live balances. Please check your password and try again.')
    } finally {
      setLiveBalanceLoading(false)
    }
  }

  const formatCurrency = (value: string) => {
    const num = parseFloat(value)
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2
    }).format(num)
  }

  const getTotalBalance = () => {
    // Calculate total from live balances if available
    if (liveBalances && liveBalances.total_usd_value) {
      return parseFloat(liveBalances.total_usd_value)
    }
    
    // Return 0 until live balances are loaded
    return 0
  }

  const getTodaysPnL = () => {
    // Calculate today's P&L from strategies with recent executions
    const todaysPnL = strategies
      .filter(s => s.recent_executions && s.recent_executions.length > 0)
      .reduce((sum, strategy) => {
        const todaysExecutions = strategy.recent_executions.filter(exec => {
          const execDate = new Date(exec.execution_timestamp)
          const today = new Date()
          return execDate.toDateString() === today.toDateString()
        })
        return sum + todaysExecutions.reduce((execSum, exec) => execSum + parseFloat(exec.amount_usd || '0'), 0)
      }, 0)
    return todaysPnL
  }

  const getRiskLevel = () => {
    const totalAllocation = parseFloat(summary.total_allocation)
    const totalBalance = getTotalBalance()

    if (totalBalance === 0) return { level: 'Low', color: 'text-emerald-400' }

    const riskRatio = totalAllocation / totalBalance

    if (riskRatio > 0.7) return { level: 'High', color: 'text-red-400' }
    if (riskRatio > 0.4) return { level: 'Medium', color: 'text-amber-400' }
    return { level: 'Low', color: 'text-emerald-400' }
  }

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'bg-emerald-500/20 text-emerald-300 border-emerald-400/30'
      case 'paused': return 'bg-amber-500/20 text-amber-300 border-amber-400/30'
      case 'completed': return 'bg-blue-500/20 text-blue-300 border-blue-400/30'
      default: return 'bg-gray-500/20 text-gray-300 border-gray-400/30'
    }
  }

  const activeConnections = connections.filter(c => c.connection_status === 'connected')
  const totalBalance = getTotalBalance()
  const todaysPnL = getTodaysPnL()
  const riskLevel = getRiskLevel()
  const recentStrategies = strategies.slice(0, 3) // Show top 3 strategies

  // SECURE: Show loading state while auth is being checked
  if (authLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-4">
            <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
              <Activity className="w-8 h-8 text-purple-300 animate-spin" />
            </div>
            <h3 className="text-xl font-semibold text-white">Loading Dashboard...</h3>
            <p className="text-gray-300">Verifying your credentials for secure access.</p>
          </div>
        </div>
      </DashboardLayout>
    )
  }

  // SECURE: Show login prompt if not authenticated
  if (!isAuthenticated) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-4 max-w-md">
            <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
              <Shield className="w-8 h-8 text-purple-300" />
            </div>
            <h3 className="text-xl font-semibold text-white">Authentication Required</h3>
            <p className="text-gray-300">Please log in to access your trading dashboard and manage your strategies.</p>
            <Link href="/login">
              <Button className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90">
                Go to Login
              </Button>
            </Link>
          </div>
        </div>
      </DashboardLayout>
    )
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Platform Overview Section */}
        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4">
          <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-200">
                Active Strategies
              </CardTitle>
              <div className="h-8 w-8 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg flex items-center justify-center">
                <Activity className="h-4 w-4 text-white" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-white">
                {loading ? '...' : summary.active_strategies}
              </div>
              <p className="text-xs text-gray-400">
                {summary.total_strategies} total strategies
              </p>
            </CardContent>
          </Card>

          <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-200">
                Connections
              </CardTitle>
              <div className="h-8 w-8 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg flex items-center justify-center">
                <DollarSign className="h-4 w-4 text-white" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-white">
                {loading ? '...' : activeConnections.length}
              </div>
              <p className="text-xs text-gray-400">
                CEX & DEX platforms
              </p>
            </CardContent>
          </Card>

          <DxyIndicator />

          <BtcPriceIndicator />
        </div>

        {/* Market Indicators Section */}
        <div className="grid gap-6 md:grid-cols-3">
          <BtcDominanceIndicator />
          <M2Indicator />
          <FearGreedIndicator />
        </div>

        {/* Strategy Type Breakdown */}
        <StrategyTypeStats
          strategies={allStrategies}
        />

        {/* Recent Strategies */}
        {summary.total_strategies > 0 && (
          <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-xl font-bold text-white">Recent Strategies</CardTitle>
                <Link href="/dashboard/strategies/unified">
                  <Button variant="outline" size="sm" className="border-[rgba(16,185,129,0.3)] bg-gradient-to-r from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] text-emerald-200 hover:bg-[rgba(16,185,129,0.2)] backdrop-blur-sm">
                    View All
                    <ArrowRight className="ml-1 h-3 w-3" />
                  </Button>
                </Link>
              </div>
            </CardHeader>
            <CardContent>
              {loading ? (
                <div className="space-y-4">
                  {[...Array(3)].map((_, i) => (
                    <div key={i} className="border border-[rgba(147,51,234,0.3)] rounded-lg p-3 bg-[rgba(147,51,234,0.05)] animate-pulse">
                      <div className="h-4 bg-gray-600 rounded mb-2"></div>
                      <div className="h-3 bg-gray-700 rounded w-3/4"></div>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="space-y-4">
                  {recentStrategies.map((strategy) => (
                    <div key={strategy.id} className="flex items-center justify-between p-3 border border-[rgba(147,51,234,0.3)] rounded-lg bg-[rgba(147,51,234,0.05)] hover:bg-[rgba(147,51,234,0.1)] transition-colors backdrop-blur-sm">
                      <div className="flex items-center space-x-3">
                        <div className="h-8 w-8 bg-gradient-to-br from-orange-500 to-orange-600 rounded-full flex items-center justify-center text-white text-xs font-bold shadow-lg">
                          {strategy.asset_symbol.slice(0, 3)}
                        </div>
                        <div>
                          <p className="font-medium text-white">{strategy.name}</p>
                          <div className="flex items-center space-x-2">
                            <Badge className={`text-xs ${getStatusColor(strategy.status)} border`}>
                              {strategy.status}
                            </Badge>
                            <span className="text-xs text-gray-400">{strategy.asset_symbol}</span>
                          </div>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className={`font-medium ${strategy.current_profit_loss && parseFloat(strategy.current_profit_loss) >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                          {strategy.current_profit_loss ? formatCurrency(strategy.current_profit_loss) : 'N/A'}
                        </p>
                        <p className="text-xs text-gray-400">
                          {formatCurrency(strategy.total_invested)} invested
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* Exchange Connection Prompt */}
        {!loading && connections.length === 0 && (
          <ConnectExchangePrompt />
        )}
      </div>
    </DashboardLayout>
  )
}
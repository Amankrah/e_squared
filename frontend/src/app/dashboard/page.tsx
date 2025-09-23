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
import { apiClient, DCAStrategy, ExchangeConnection, WalletBalance } from "@/lib/api"
import { useAuth } from "@/contexts/auth-context"
import Link from "next/link"

export default function Dashboard() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [strategies, setStrategies] = useState<DCAStrategy[]>([])
  const [connections, setConnections] = useState<ExchangeConnection[]>([])
  const [balances, setBalances] = useState<WalletBalance[]>([])
  const [loading, setLoading] = useState(false)
  const [liveBalanceLoading, setLiveBalanceLoading] = useState(false)
  const [summary, setSummary] = useState({
    total_allocation: "0",
    total_invested: "0",
    total_profit_loss: "0",
    active_strategies: 0
  })

  // SECURE: Only load dashboard data when authenticated
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadDashboardData()
    } else if (!authLoading && !isAuthenticated) {
      // User is not authenticated, clear any existing data
      setStrategies([])
      setConnections([])
      setBalances([])
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
      const [strategiesRes, connectionsRes, balancesRes] = await Promise.all([
        apiClient.getDCAStrategies().catch(() => ({ strategies: [], total_allocation: "0", total_invested: "0", total_profit_loss: "0", active_strategies: 0 })),
        apiClient.getExchangeConnections().catch(() => ({ connections: [], message: '' })),
        // For dashboard, use the stored balances endpoint since live balances require password
        // Users can get live balances by clicking "Load Live Balances" in the exchanges page
        apiClient.getAllUserBalances().catch(() => ({ balances: [], message: '' }))
      ])

      setStrategies(strategiesRes.strategies || [])
      setConnections(connectionsRes.connections || [])
      setBalances(balancesRes.balances || [])
      setSummary({
        total_allocation: strategiesRes.total_allocation,
        total_invested: strategiesRes.total_invested,
        total_profit_loss: strategiesRes.total_profit_loss,
        active_strategies: strategiesRes.active_strategies
      })
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
      // Trigger sync for all connections to update stored balances with live data
      // This will prompt user for password when needed
      const password = prompt("Enter your password to sync live balances:")
      if (!password) {
        setLiveBalanceLoading(false)
        return
      }

      // Sync each connection to update stored balances with live Binance prices
      for (const connection of connections) {
        try {
          await apiClient.syncExchangeConnection(connection.id, password)
        } catch (error) {
          console.error(`Failed to sync connection ${connection.id}:`, error)
        }
      }

      // After syncing, reload the dashboard data to get updated balances
      await loadDashboardData()
    } catch (error) {
      console.error('Failed to load live balances:', error)
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
    // Calculate total from balances array
    const balanceTotal = balances
      .filter(b => b.usd_value)
      .reduce((sum, balance) => sum + parseFloat(balance.usd_value!), 0)

    // If we have no balances or zero total, show a placeholder until live balances are loaded
    return balanceTotal
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

  // Show connect exchange prompt if user has no exchange connections
  if (!loading && connections.length === 0) {
    return (
      <DashboardLayout>
        <div className="max-w-2xl mx-auto mt-8">
          <ConnectExchangePrompt />
        </div>
      </DashboardLayout>
    )
  }

  return (
    <DashboardLayout>
      {/* Overview Cards */}
      <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-4 mb-8">
        <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-200">
              Total Balance
            </CardTitle>
            <div className="h-8 w-8 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-lg flex items-center justify-center">
              <DollarSign className="h-4 w-4 text-white" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-white">
              {loading ? '...' : formatCurrency(totalBalance.toString())}
            </div>
            <div className="flex items-center justify-between mt-2">
              <p className="text-xs text-gray-400">
                Across {connections.length} exchange{connections.length !== 1 ? 's' : ''}
              </p>
              <Button
                size="sm"
                variant="outline"
                onClick={loadLiveBalances}
                disabled={liveBalanceLoading || connections.length === 0}
                className="text-xs h-6 px-2 border-emerald-400/30 text-emerald-300 hover:bg-emerald-500/20"
              >
                {liveBalanceLoading ? (
                  <Activity className="w-3 h-3 animate-spin" />
                ) : (
                  'Live'
                )}
              </Button>
            </div>
          </CardContent>
        </Card>

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
              {strategies.filter(s => parseFloat(s.current_profit_loss || '0') > 0).length} profitable, {strategies.filter(s => s.status === 'paused').length} paused
            </p>
          </CardContent>
        </Card>

        <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-200">
              Total P&L
            </CardTitle>
            <div className="h-8 w-8 bg-gradient-to-br from-emerald-500 to-green-600 rounded-lg flex items-center justify-center">
              <TrendingUp className="h-4 w-4 text-white" />
            </div>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${parseFloat(summary.total_profit_loss) >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
              {loading ? '...' : formatCurrency(summary.total_profit_loss)}
            </div>
            <p className="text-xs text-gray-400">
              From DCA strategies
            </p>
          </CardContent>
        </Card>

        <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-200">
              Risk Level
            </CardTitle>
            <div className="h-8 w-8 bg-gradient-to-br from-amber-500 to-orange-600 rounded-lg flex items-center justify-center">
              <AlertTriangle className="h-4 w-4 text-white" />
            </div>
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${riskLevel.color}`}>
              {loading ? '...' : riskLevel.level}
            </div>
            <p className="text-xs text-gray-400">
              Based on allocation ratio
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Strategy Builder Section */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="lg:col-span-1 border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
          <CardHeader>
            <CardTitle className="text-xl font-bold text-white">DCA Strategy Builder</CardTitle>
            <CardDescription className="text-gray-300">
              Create and manage your dollar-cost averaging strategies
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid gap-2">
              <Link href="/dashboard/strategies">
                <Button className="w-full bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white shadow-lg">
                  <PlusCircle className="mr-2 h-4 w-4" />
                  Create New Strategy
                </Button>
              </Link>
              <div className="grid grid-cols-2 gap-2">
                <Link href="/dashboard/strategies?tab=templates">
                  <Button variant="outline" size="sm" className="w-full border-[rgba(59,130,246,0.3)] bg-gradient-to-r from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] text-blue-200 hover:bg-[rgba(59,130,246,0.2)] backdrop-blur-sm">
                    <Target className="mr-1 h-3 w-3" />
                    Templates
                  </Button>
                </Link>
                <Button variant="outline" size="sm" className="border-[rgba(147,51,234,0.3)] bg-gradient-to-r from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] text-purple-200 hover:bg-[rgba(147,51,234,0.2)] backdrop-blur-sm">
                  Backtest
                </Button>
              </div>
            </div>

            {/* Strategy Overview */}
            <div className="border border-[rgba(147,51,234,0.3)] rounded-lg p-4 space-y-3 bg-[rgba(147,51,234,0.05)] backdrop-blur-sm">
              <div className="flex items-center justify-between">
                <h4 className="font-medium text-white">Strategy Overview</h4>
                <Badge className="bg-emerald-500/20 text-emerald-300 border-emerald-400/30">
                  {summary.active_strategies} Active
                </Badge>
              </div>
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <p className="text-gray-400">Total Allocated</p>
                  <p className="font-semibold text-white">
                    {loading ? '...' : formatCurrency(summary.total_allocation)}
                  </p>
                </div>
                <div>
                  <p className="text-gray-400">Invested</p>
                  <p className="font-semibold text-white">
                    {loading ? '...' : formatCurrency(summary.total_invested)}
                  </p>
                </div>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">Performance:</span>
                <span className={`font-medium ${parseFloat(summary.total_profit_loss) >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                  {loading ? '...' : formatCurrency(summary.total_profit_loss)}
                </span>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Active Strategies Monitor */}
        <Card className="lg:col-span-1 border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="text-xl font-bold text-white">Active Strategies</CardTitle>
                <CardDescription className="text-gray-300">
                  Monitor your live DCA strategies
                </CardDescription>
              </div>
              <Link href="/dashboard/strategies">
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
            ) : recentStrategies.length === 0 ? (
              <div className="text-center py-8">
                <Target className="w-12 h-12 text-purple-300 mx-auto mb-4" />
                <p className="text-gray-300 mb-4">No strategies created yet</p>
                <Link href="/dashboard/strategies">
                  <Button size="sm" className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90">
                    <PlusCircle className="w-3 h-3 mr-1" />
                    Create First Strategy
                  </Button>
                </Link>
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
      </div>

      {/* Quick Actions Section */}
      <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Link href="/dashboard/exchanges">
          <Card className="border-2 border-[rgba(59,130,246,0.3)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300 cursor-pointer group">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-3 bg-gradient-to-br from-blue-500/30 to-cyan-500/30 rounded-xl backdrop-blur-sm border border-blue-400/20">
                    <Shield className="w-6 h-6 text-blue-200" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-white">Connect Exchange</h3>
                    <p className="text-sm text-gray-300">Securely link your trading accounts</p>
                  </div>
                </div>
                <ArrowRight className="w-5 h-5 text-blue-200 group-hover:translate-x-1 transition-transform" />
              </div>
            </CardContent>
          </Card>
        </Link>

        <Link href="/dashboard/strategies">
          <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300 cursor-pointer group">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-xl backdrop-blur-sm border border-purple-400/20">
                    <TrendingUp className="w-6 h-6 text-purple-200" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-white">Build Strategy</h3>
                    <p className="text-sm text-gray-300">Create intelligent trading algorithms</p>
                  </div>
                </div>
                <ArrowRight className="w-5 h-5 text-purple-200 group-hover:translate-x-1 transition-transform" />
              </div>
            </CardContent>
          </Card>
        </Link>

        <Link href="/dashboard/settings">
          <Card className="border-2 border-[rgba(16,185,129,0.3)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl shadow-lg hover:shadow-xl transition-all duration-300 cursor-pointer group">
            <CardContent className="p-6">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <div className="p-3 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-xl backdrop-blur-sm border border-emerald-400/20">
                    <Settings className="w-6 h-6 text-emerald-200" />
                  </div>
                  <div>
                    <h3 className="text-lg font-semibold text-white">Settings</h3>
                    <p className="text-sm text-gray-300">Customize your trading experience</p>
                  </div>
                </div>
                <ArrowRight className="w-5 h-5 text-emerald-200 group-hover:translate-x-1 transition-transform" />
              </div>
            </CardContent>
          </Card>
        </Link>
      </div>
    </DashboardLayout>
  )
}
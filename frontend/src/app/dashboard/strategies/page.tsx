"use client"

import { useState, useEffect } from "react"
import { TrendingUp, Plus, BarChart3, Settings, Play, Pause, Edit, Clock, Target, DollarSign, Activity } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Badge } from "@/components/ui/badge"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { apiClient, DCAStrategy, DCAStrategiesResponse, StrategyTemplate } from "@/lib/api"
import Link from "next/link"

export default function StrategiesPage() {
  const [strategies, setStrategies] = useState<DCAStrategy[]>([])
  const [templates, setTemplates] = useState<StrategyTemplate[]>([])
  const [loading, setLoading] = useState(true)
  const [activeTab, setActiveTab] = useState("strategies")
  const [summary, setSummary] = useState({
    total_allocation: "0",
    total_invested: "0",
    total_profit_loss: "0",
    active_strategies: 0
  })

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      const [strategiesRes, templatesRes] = await Promise.all([
        apiClient.getDCAStrategies(),
        apiClient.getStrategyTemplates()
      ])

      setStrategies(strategiesRes.strategies)
      // Handle both array and object response formats
      if (Array.isArray(templatesRes)) {
        setTemplates(templatesRes)
      } else if (templatesRes && templatesRes.templates) {
        setTemplates(templatesRes.templates)
      } else {
        setTemplates([])
      }
      setSummary({
        total_allocation: strategiesRes.total_allocation,
        total_invested: strategiesRes.total_invested,
        total_profit_loss: strategiesRes.total_profit_loss,
        active_strategies: strategiesRes.active_strategies
      })
    } catch (error) {
      console.error('Failed to load strategies:', error)
      setTemplates([]) // Ensure templates is always an array
    } finally {
      setLoading(false)
    }
  }

  const handleStrategyAction = async (strategyId: string, action: 'pause' | 'resume') => {
    try {
      if (action === 'pause') {
        await apiClient.pauseDCAStrategy(strategyId)
      } else {
        await apiClient.resumeDCAStrategy(strategyId)
      }
      await loadData() // Refresh data
    } catch (error) {
      console.error(`Failed to ${action} strategy:`, error)
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

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'bg-emerald-500/20 text-emerald-300 border-emerald-400/30'
      case 'paused': return 'bg-amber-500/20 text-amber-300 border-amber-400/30'
      case 'completed': return 'bg-blue-500/20 text-blue-300 border-blue-400/30'
      default: return 'bg-gray-500/20 text-gray-300 border-gray-400/30'
    }
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Page Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                DCA Strategies
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Build, test, and deploy intelligent dollar-cost averaging strategies
            </p>
          </div>

          <div className="flex items-center space-x-4">
            <Link href="/dashboard/strategies/create">
              <Button className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white shadow-lg">
                <Plus className="w-4 h-4 mr-2" />
                Create Strategy
              </Button>
            </Link>
          </div>
        </div>

        {/* Summary Stats */}
        <div className="grid gap-4 md:grid-cols-4 mb-6">
          <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
            <CardContent className="p-4">
              <div className="flex items-center space-x-3">
                <DollarSign className="w-8 h-8 text-emerald-300" />
                <div>
                  <p className="text-xs text-gray-400 uppercase tracking-wide">Total Allocation</p>
                  <p className="text-xl font-bold text-white">{formatCurrency(summary.total_allocation)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
            <CardContent className="p-4">
              <div className="flex items-center space-x-3">
                <Target className="w-8 h-8 text-purple-300" />
                <div>
                  <p className="text-xs text-gray-400 uppercase tracking-wide">Invested</p>
                  <p className="text-xl font-bold text-white">{formatCurrency(summary.total_invested)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="border-2 border-[rgba(59,130,246,0.2)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl">
            <CardContent className="p-4">
              <div className="flex items-center space-x-3">
                <TrendingUp className="w-8 h-8 text-blue-300" />
                <div>
                  <p className="text-xs text-gray-400 uppercase tracking-wide">P&L</p>
                  <p className={`text-xl font-bold ${parseFloat(summary.total_profit_loss) >= 0 ? 'text-emerald-300' : 'text-red-300'}`}>
                    {formatCurrency(summary.total_profit_loss)}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="border-2 border-[rgba(244,63,94,0.2)] bg-gradient-to-br from-[rgba(244,63,94,0.1)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl">
            <CardContent className="p-4">
              <div className="flex items-center space-x-3">
                <Activity className="w-8 h-8 text-pink-300" />
                <div>
                  <p className="text-xs text-gray-400 uppercase tracking-wide">Active</p>
                  <p className="text-xl font-bold text-white">{summary.active_strategies}</p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Content */}
        <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
          <TabsList className="grid grid-cols-2 w-fit h-12 rounded-xl border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-1">
            <TabsTrigger value="strategies" className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium">
              <Activity className="w-4 h-4 mr-2" />
              My Strategies ({strategies.length})
            </TabsTrigger>
            <TabsTrigger value="templates" className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium">
              <BarChart3 className="w-4 h-4 mr-2" />
              Templates ({templates.length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value="strategies" className="space-y-6">
            {loading ? (
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {[...Array(6)].map((_, i) => (
                  <Card key={i} className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl animate-pulse">
                    <CardContent className="p-6">
                      <div className="h-4 bg-gray-600 rounded mb-4"></div>
                      <div className="h-3 bg-gray-700 rounded mb-2"></div>
                      <div className="h-3 bg-gray-700 rounded w-3/4"></div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : strategies.length === 0 ? (
              <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                <CardContent className="p-12 text-center">
                  <TrendingUp className="w-16 h-16 text-purple-300 mx-auto mb-4" />
                  <h3 className="text-xl font-semibold text-white mb-2">No Strategies Yet</h3>
                  <p className="text-gray-300 mb-6">Get started by creating your first DCA strategy</p>
                  <Link href="/dashboard/strategies/create">
                    <Button className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90">
                      <Plus className="w-4 h-4 mr-2" />
                      Create Your First Strategy
                    </Button>
                  </Link>
                </CardContent>
              </Card>
            ) : (
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                {strategies.map((strategy) => (
                  <Card key={strategy.id} className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl hover:border-[rgba(147,51,234,0.4)] transition-all">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-lg font-semibold text-white truncate">
                          {strategy.name}
                        </CardTitle>
                        <Badge className={`${getStatusColor(strategy.status)} border`}>
                          {strategy.status}
                        </Badge>
                      </div>
                      <div className="flex items-center space-x-2 text-sm text-gray-300">
                        <span>{strategy.asset_symbol}</span>
                        <span>•</span>
                        <span>{strategy.strategy_type}</span>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="grid grid-cols-2 gap-4 text-sm">
                        <div>
                          <p className="text-gray-400">Allocation</p>
                          <p className="font-semibold text-white">{formatCurrency(strategy.total_allocation)}</p>
                        </div>
                        <div>
                          <p className="text-gray-400">Invested</p>
                          <p className="font-semibold text-white">{formatCurrency(strategy.total_invested)}</p>
                        </div>
                        <div>
                          <p className="text-gray-400">P&L</p>
                          <p className={`font-semibold ${strategy.current_profit_loss && parseFloat(strategy.current_profit_loss) >= 0 ? 'text-emerald-300' : 'text-red-300'}`}>
                            {strategy.current_profit_loss ? formatCurrency(strategy.current_profit_loss) : 'N/A'}
                          </p>
                        </div>
                        <div>
                          <p className="text-gray-400">Next Run</p>
                          <p className="font-semibold text-white text-xs">
                            {strategy.next_execution_at ? new Date(strategy.next_execution_at).toLocaleDateString() : 'N/A'}
                          </p>
                        </div>
                      </div>

                      <div className="flex space-x-2">
                        <Button
                          size="sm"
                          variant="outline"
                          className="flex-1 border-[rgba(147,51,234,0.3)] text-purple-200 hover:bg-[rgba(147,51,234,0.2)]"
                          asChild
                        >
                          <Link href={`/dashboard/strategies/${strategy.id}`}>
                            <Edit className="w-3 h-3 mr-1" />
                            Edit
                          </Link>
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          className={`border-[rgba(16,185,129,0.3)] text-emerald-200 hover:bg-[rgba(16,185,129,0.2)] ${strategy.status === 'active' ? 'border-[rgba(244,63,94,0.3)] text-red-200 hover:bg-[rgba(244,63,94,0.2)]' : ''}`}
                          onClick={() => handleStrategyAction(strategy.id, strategy.status === 'active' ? 'pause' : 'resume')}
                        >
                          {strategy.status === 'active' ? (
                            <><Pause className="w-3 h-3 mr-1" />Pause</>
                          ) : (
                            <><Play className="w-3 h-3 mr-1" />Resume</>
                          )}
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </TabsContent>

          <TabsContent value="templates" className="space-y-6">
            {templates.length === 0 ? (
              <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                <CardContent className="p-12 text-center">
                  <BarChart3 className="w-16 h-16 text-purple-300 mx-auto mb-4" />
                  <h3 className="text-xl font-semibold text-white mb-2">No Templates Available</h3>
                  <p className="text-gray-300">Strategy templates will be loaded from the backend</p>
                </CardContent>
              </Card>
            ) : (
              <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
                {templates.map((template) => (
                  <Card key={template.id} className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl hover:border-[rgba(147,51,234,0.4)] transition-all">
                    <CardHeader>
                      <CardTitle className="text-lg font-semibold text-white">{template.name}</CardTitle>
                      <CardDescription className="text-gray-300">{template.description}</CardDescription>
                      <div className="flex space-x-2">
                        <Badge className="bg-blue-500/20 text-blue-300 border-blue-400/30">
                          {template.risk_level}
                        </Badge>
                        <Badge className="bg-purple-500/20 text-purple-300 border-purple-400/30">
                          {template.category}
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="text-sm space-y-2">
                        <div className="flex justify-between">
                          <span className="text-gray-400">Time Horizon:</span>
                          <span className="text-white">{template.time_horizon}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-400">Complexity:</span>
                          <span className="text-white">{template.complexity}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-400">Recommended:</span>
                          <span className="text-white">{formatCurrency(template.recommended_allocation.recommended_usd)}</span>
                        </div>
                      </div>

                      <div className="flex space-x-2">
                        <Button
                          size="sm"
                          className="flex-1 bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90"
                          asChild
                        >
                          <Link href={`/dashboard/strategies/create?template=${template.id}`}>
                            <Plus className="w-3 h-3 mr-1" />
                            Use Template
                          </Link>
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          className="border-[rgba(59,130,246,0.3)] text-blue-200 hover:bg-[rgba(59,130,246,0.2)]"
                          asChild
                        >
                          <Link href={`/dashboard/strategies/backtest?template=${template.id}`}>
                            <BarChart3 className="w-3 h-3 mr-1" />
                            Backtest
                          </Link>
                        </Button>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
}
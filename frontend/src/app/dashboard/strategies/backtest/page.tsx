"use client"

import { useState, useEffect } from "react"
import { useSearchParams, useRouter } from "next/navigation"
import { TrendingUp, Calendar, DollarSign, BarChart3, ArrowLeft, Play, Download, RefreshCw } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { PerformanceChart } from "@/components/charts/performance-chart"
import { apiClient, StrategyTemplate } from "@/lib/api"
import { useToast } from "@/hooks/use-toast"
import Link from "next/link"

interface BacktestResult {
  total_return: number
  total_return_percentage: number
  sharpe_ratio: number
  max_drawdown: number
  max_drawdown_percentage: number
  total_trades: number
  win_rate: number
  final_balance: number
  initial_balance: number
  volatility: number
  start_date: string
  end_date: string
  daily_returns: Array<{
    date: string
    portfolio_value: number
    return_percentage: number
  }>
}

interface BacktestResponse {
  template_id: string
  template_name: string
  backtest_result: BacktestResult
  user_id: string
  created_at: string
}

export default function BacktestPage() {
  const searchParams = useSearchParams()
  const router = useRouter()
  const { toast } = useToast()

  const [template, setTemplate] = useState<StrategyTemplate | null>(null)
  const [templates, setTemplates] = useState<StrategyTemplate[]>([])
  const [selectedTemplateId, setSelectedTemplateId] = useState(searchParams.get('template') || '')
  const [loading, setLoading] = useState(true)
  const [backtesting, setBacktesting] = useState(false)
  const [result, setResult] = useState<BacktestResponse | null>(null)

  // Backtest parameters
  const [symbol, setSymbol] = useState('BTCUSDT')
  const [interval, setInterval] = useState('1h')
  const [startDate, setStartDate] = useState('2023-01-01')
  const [endDate, setEndDate] = useState('2023-12-31')
  const [initialBalance, setInitialBalance] = useState('10000')
  const [templateParameters, setTemplateParameters] = useState<Record<string, any>>({})

  useEffect(() => {
    loadTemplates()
  }, [])

  useEffect(() => {
    if (selectedTemplateId && templates.length > 0) {
      const foundTemplate = templates.find(t => t.id === selectedTemplateId)
      setTemplate(foundTemplate || null)

      // Set default parameters based on template
      if (foundTemplate) {
        setTemplateParameters(foundTemplate.default_parameters || {})
      }
    }
  }, [selectedTemplateId, templates])

  const loadTemplates = async () => {
    try {
      const response = await apiClient.getStrategyTemplates()
      const templatesData = Array.isArray(response) ? response : response.templates || []
      setTemplates(templatesData)
    } catch (error) {
      console.error('Failed to load templates:', error)
      toast({
        title: "Error",
        description: "Failed to load strategy templates",
        variant: "destructive",
      })
    } finally {
      setLoading(false)
    }
  }

  const runBacktest = async () => {
    if (!selectedTemplateId) {
      toast({
        title: "Template Required",
        description: "Please select a strategy template to backtest",
        variant: "destructive",
      })
      return
    }

    setBacktesting(true)
    try {
      const response = await fetch(`/api/v1/strategy-templates/${selectedTemplateId}/backtest`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
        body: JSON.stringify({
          template_id: selectedTemplateId,
          symbol,
          interval,
          start_time: new Date(startDate).toISOString(),
          end_time: new Date(endDate).toISOString(),
          initial_balance: parseFloat(initialBalance),
          template_parameters: templateParameters,
        }),
      })

      if (!response.ok) {
        throw new Error('Failed to run backtest')
      }

      const data: BacktestResponse = await response.json()
      setResult(data)

      toast({
        title: "Backtest Complete",
        description: `Backtest completed for ${data.template_name}`,
      })
    } catch (error) {
      console.error('Backtest failed:', error)
      toast({
        title: "Backtest Failed",
        description: "Failed to run backtest. Please try again.",
        variant: "destructive",
      })
    } finally {
      setBacktesting(false)
    }
  }

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2
    }).format(value)
  }

  const formatPercentage = (value: number) => {
    return `${(value * 100).toFixed(2)}%`
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Header */}
        <div className="flex items-center space-x-4">
          <Link href="/dashboard/strategies">
            <Button variant="outline" size="sm" className="border-[rgba(147,51,234,0.3)] text-purple-200 hover:bg-[rgba(147,51,234,0.2)]">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Strategies
            </Button>
          </Link>
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Strategy Backtesting
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Test strategy performance on historical data
            </p>
          </div>
        </div>

        <div className="grid gap-8 lg:grid-cols-3">
          {/* Configuration Panel */}
          <div className="lg:col-span-1 space-y-6">
            <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
              <CardHeader>
                <CardTitle className="text-white">Backtest Configuration</CardTitle>
                <CardDescription>Configure your backtest parameters</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {/* Template Selection */}
                <div className="space-y-2">
                  <Label className="text-white">Strategy Template</Label>
                  <Select value={selectedTemplateId} onValueChange={setSelectedTemplateId}>
                    <SelectTrigger>
                      <SelectValue placeholder="Select a template" />
                    </SelectTrigger>
                    <SelectContent>
                      {templates.map((template) => (
                        <SelectItem key={template.id} value={template.id}>
                          {template.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {/* Symbol */}
                <div className="space-y-2">
                  <Label className="text-white">Trading Pair</Label>
                  <Select value={symbol} onValueChange={setSymbol}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="BTCUSDT">BTC/USDT</SelectItem>
                      <SelectItem value="ETHUSDT">ETH/USDT</SelectItem>
                      <SelectItem value="ADAUSDT">ADA/USDT</SelectItem>
                      <SelectItem value="DOTUSDT">DOT/USDT</SelectItem>
                      <SelectItem value="SOLUSDT">SOL/USDT</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Timeframe */}
                <div className="space-y-2">
                  <Label className="text-white">Timeframe</Label>
                  <Select value={interval} onValueChange={setInterval}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1h">1 Hour</SelectItem>
                      <SelectItem value="4h">4 Hours</SelectItem>
                      <SelectItem value="1d">1 Day</SelectItem>
                      <SelectItem value="1w">1 Week</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Date Range */}
                <div className="grid grid-cols-2 gap-2">
                  <div className="space-y-2">
                    <Label className="text-white">Start Date</Label>
                    <Input
                      type="date"
                      value={startDate}
                      onChange={(e) => setStartDate(e.target.value)}
                      className="bg-black/20 border-[rgba(147,51,234,0.3)] text-white"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label className="text-white">End Date</Label>
                    <Input
                      type="date"
                      value={endDate}
                      onChange={(e) => setEndDate(e.target.value)}
                      className="bg-black/20 border-[rgba(147,51,234,0.3)] text-white"
                    />
                  </div>
                </div>

                {/* Initial Balance */}
                <div className="space-y-2">
                  <Label className="text-white">Initial Balance (USD)</Label>
                  <Input
                    type="number"
                    value={initialBalance}
                    onChange={(e) => setInitialBalance(e.target.value)}
                    className="bg-black/20 border-[rgba(147,51,234,0.3)] text-white"
                    min="100"
                    step="100"
                  />
                </div>

                {/* Run Backtest Button */}
                <Button
                  onClick={runBacktest}
                  disabled={backtesting || !selectedTemplateId}
                  className="w-full bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90"
                >
                  {backtesting ? (
                    <>
                      <RefreshCw className="w-4 h-4 mr-2 animate-spin" />
                      Running Backtest...
                    </>
                  ) : (
                    <>
                      <Play className="w-4 h-4 mr-2" />
                      Run Backtest
                    </>
                  )}
                </Button>
              </CardContent>
            </Card>

            {/* Template Info */}
            {template && (
              <Card className="border-2 border-[rgba(59,130,246,0.2)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl">
                <CardHeader>
                  <CardTitle className="text-white">{template.name}</CardTitle>
                  <CardDescription>{template.description}</CardDescription>
                </CardHeader>
                <CardContent className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">Risk Level:</span>
                    <span className="text-white">{template.risk_level}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">Category:</span>
                    <span className="text-white">{template.category}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">Complexity:</span>
                    <span className="text-white">{template.complexity}</span>
                  </div>
                </CardContent>
              </Card>
            )}
          </div>

          {/* Results Panel */}
          <div className="lg:col-span-2 space-y-6">
            {result ? (
              <>
                {/* Performance Metrics */}
                <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                  <Card className="border-2 border-[rgba(16,185,129,0.2)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
                    <CardContent className="p-4">
                      <div className="flex items-center space-x-3">
                        <TrendingUp className="w-8 h-8 text-emerald-300" />
                        <div>
                          <p className="text-xs text-gray-400 uppercase tracking-wide">Total Return</p>
                          <p className="text-xl font-bold text-white">
                            {formatPercentage(result.backtest_result.total_return_percentage)}
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                    <CardContent className="p-4">
                      <div className="flex items-center space-x-3">
                        <BarChart3 className="w-8 h-8 text-purple-300" />
                        <div>
                          <p className="text-xs text-gray-400 uppercase tracking-wide">Sharpe Ratio</p>
                          <p className="text-xl font-bold text-white">
                            {result.backtest_result.sharpe_ratio.toFixed(2)}
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="border-2 border-[rgba(244,63,94,0.2)] bg-gradient-to-br from-[rgba(244,63,94,0.1)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl">
                    <CardContent className="p-4">
                      <div className="flex items-center space-x-3">
                        <TrendingUp className="w-8 h-8 text-red-300 rotate-180" />
                        <div>
                          <p className="text-xs text-gray-400 uppercase tracking-wide">Max Drawdown</p>
                          <p className="text-xl font-bold text-white">
                            {formatPercentage(result.backtest_result.max_drawdown_percentage)}
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>

                  <Card className="border-2 border-[rgba(59,130,246,0.2)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl">
                    <CardContent className="p-4">
                      <div className="flex items-center space-x-3">
                        <DollarSign className="w-8 h-8 text-blue-300" />
                        <div>
                          <p className="text-xs text-gray-400 uppercase tracking-wide">Final Balance</p>
                          <p className="text-xl font-bold text-white">
                            {formatCurrency(result.backtest_result.final_balance)}
                          </p>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                </div>

                {/* Detailed Results */}
                <Card className="border-2 border-[rgba(147,51,234,0.2)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle className="text-white">Backtest Results</CardTitle>
                        <CardDescription>{result.template_name} • {result.backtest_result.start_date} to {result.backtest_result.end_date}</CardDescription>
                      </div>
                      <Button variant="outline" size="sm" className="border-[rgba(147,51,234,0.3)] text-purple-200 hover:bg-[rgba(147,51,234,0.2)]">
                        <Download className="w-4 h-4 mr-2" />
                        Export
                      </Button>
                    </div>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Total Trades</p>
                        <p className="text-lg font-semibold text-white">{result.backtest_result.total_trades}</p>
                      </div>
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Win Rate</p>
                        <p className="text-lg font-semibold text-white">{formatPercentage(result.backtest_result.win_rate)}</p>
                      </div>
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Volatility</p>
                        <p className="text-lg font-semibold text-white">{formatPercentage(result.backtest_result.volatility)}</p>
                      </div>
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Initial Balance</p>
                        <p className="text-lg font-semibold text-white">{formatCurrency(result.backtest_result.initial_balance)}</p>
                      </div>
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Total Return (USD)</p>
                        <p className="text-lg font-semibold text-white">{formatCurrency(result.backtest_result.total_return)}</p>
                      </div>
                      <div className="space-y-2">
                        <p className="text-sm text-gray-400">Max Drawdown (USD)</p>
                        <p className="text-lg font-semibold text-white">{formatCurrency(result.backtest_result.max_drawdown)}</p>
                      </div>
                    </div>

                    {/* Performance Chart */}
                    <div className="bg-black/20 border border-[rgba(147,51,234,0.3)] rounded-lg p-6">
                      <div className="flex items-center justify-between mb-4">
                        <h3 className="text-white font-semibold">Portfolio Performance</h3>
                        <div className="text-sm text-gray-400">
                          {result.backtest_result.daily_returns.length} data points
                        </div>
                      </div>
                      <PerformanceChart
                        data={result.backtest_result.daily_returns}
                        initialBalance={result.backtest_result.initial_balance}
                      />
                    </div>
                  </CardContent>
                </Card>
              </>
            ) : (
              <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                <CardContent className="p-12 text-center">
                  <BarChart3 className="w-16 h-16 text-purple-300 mx-auto mb-4" />
                  <h3 className="text-xl font-semibold text-white mb-2">Ready to Backtest</h3>
                  <p className="text-gray-300">Configure your parameters and run a backtest to see strategy performance</p>
                </CardContent>
              </Card>
            )}
          </div>
        </div>
      </div>
    </DashboardLayout>
  )
}
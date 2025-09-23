"use client"

import { useState, useEffect } from "react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import {
  Wallet,
  TrendingUp,
  TrendingDown,
  BarChart3,
  Activity,
  Eye,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  DollarSign,
  Coins
} from "lucide-react"
import { apiClient, type AccountResponse, type SpotAccount, type MarginAccount, type FuturesAccount } from "@/lib/api"
import { formatCurrency, formatNumber } from "@/lib/utils"

interface ExchangeAccountsPageProps {}

export default function ExchangeAccountsPage({}: ExchangeAccountsPageProps) {
  const [accounts, setAccounts] = useState<AccountResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [selectedTab, setSelectedTab] = useState("overview")

  useEffect(() => {
    fetchAccounts()
  }, [])

  const fetchAccounts = async () => {
    try {
      setLoading(true)
      setError(null)
      const data = await apiClient.getAllExchangeAccounts()
      setAccounts(data)
    } catch (err: any) {
      setError(err.message || "Failed to fetch exchange accounts")
    } finally {
      setLoading(false)
    }
  }

  const getTotalValues = () => {
    const totals = accounts.reduce(
      (acc, account) => {
        const usdValue = parseFloat(account.accounts.total_usd_value || "0")
        const btcValue = parseFloat(account.accounts.total_btc_value || "0")
        return {
          usd: acc.usd + usdValue,
          btc: acc.btc + btcValue
        }
      },
      { usd: 0, btc: 0 }
    )
    return totals
  }

  const getWalletTypeBadge = (type: string) => {
    const colors = {
      spot: "bg-blue-500/20 text-blue-300 border-blue-400/30",
      margin: "bg-orange-500/20 text-orange-300 border-orange-400/30",
      futures: "bg-purple-500/20 text-purple-300 border-purple-400/30",
      futures_coin: "bg-yellow-500/20 text-yellow-300 border-yellow-400/30"
    }
    return colors[type as keyof typeof colors] || "bg-gray-500/20 text-gray-300 border-gray-400/30"
  }

  const renderSpotAccount = (account: SpotAccount, exchangeName: string) => (
    <Card className="bg-gradient-to-br from-blue-500/10 to-blue-600/5 border-blue-400/20">
      <CardHeader>
        <CardTitle className="flex items-center space-x-2 text-blue-200">
          <Wallet className="w-5 h-5" />
          <span>Spot Trading</span>
          <Badge className={getWalletTypeBadge("spot")}>SPOT</Badge>
        </CardTitle>
        <CardDescription className="text-gray-300">
          Available for immediate trading
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Total Value</p>
            <p className="text-2xl font-bold text-blue-200">
              {formatCurrency(parseFloat(account.total_usd_value || "0"))}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Trading Status</p>
            <div className="flex items-center space-x-2">
              {account.can_trade ? (
                <CheckCircle className="w-4 h-4 text-green-400" />
              ) : (
                <AlertTriangle className="w-4 h-4 text-red-400" />
              )}
              <span className={account.can_trade ? "text-green-300" : "text-red-300"}>
                {account.can_trade ? "Enabled" : "Disabled"}
              </span>
            </div>
          </div>
        </div>

        {account.balances.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium text-gray-300">Asset Balances</h4>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {account.balances
                .filter(balance => parseFloat(balance.total) > 0)
                .map((balance, index) => (
                  <div key={index} className="flex items-center justify-between p-2 bg-white/5 rounded-lg">
                    <div className="flex items-center space-x-2">
                      <span className="font-medium text-white">{balance.asset}</span>
                    </div>
                    <div className="text-right">
                      <p className="text-sm font-medium text-white">{formatNumber(parseFloat(balance.total))}</p>
                      {balance.usd_value && (
                        <p className="text-xs text-gray-400">{formatCurrency(parseFloat(balance.usd_value))}</p>
                      )}
                    </div>
                  </div>
                ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )

  const renderFuturesAccount = (account: FuturesAccount, exchangeName: string) => (
    <Card className="bg-gradient-to-br from-purple-500/10 to-purple-600/5 border-purple-400/20">
      <CardHeader>
        <CardTitle className="flex items-center space-x-2 text-purple-200">
          <BarChart3 className="w-5 h-5" />
          <span>Futures Trading</span>
          <Badge className={getWalletTypeBadge("futures")}>
            {account.account_type.toUpperCase()}
          </Badge>
        </CardTitle>
        <CardDescription className="text-gray-300">
          Leveraged trading with margin
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-3 gap-4">
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Wallet Balance</p>
            <p className="text-lg font-bold text-purple-200">
              {formatCurrency(parseFloat(account.total_wallet_balance))}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Unrealized PnL</p>
            <p className={`text-lg font-bold ${
              parseFloat(account.total_unrealized_pnl) >= 0 ? "text-green-400" : "text-red-400"
            }`}>
              {parseFloat(account.total_unrealized_pnl) >= 0 ? "+" : ""}
              {formatCurrency(parseFloat(account.total_unrealized_pnl))}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Available</p>
            <p className="text-lg font-bold text-purple-200">
              {formatCurrency(parseFloat(account.available_balance))}
            </p>
          </div>
        </div>

        {account.positions.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium text-gray-300">Open Positions</h4>
            <div className="space-y-2 max-h-40 overflow-y-auto">
              {account.positions.map((position, index) => (
                <div key={index} className="p-3 bg-white/5 rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium text-white">{position.symbol}</span>
                    <Badge className={
                      position.position_side === 'long'
                        ? "bg-green-500/20 text-green-300 border-green-400/30"
                        : "bg-red-500/20 text-red-300 border-red-400/30"
                    }>
                      {position.position_side.toUpperCase()} {position.leverage}x
                    </Badge>
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-gray-400">Size: </span>
                      <span className="text-white">{formatNumber(parseFloat(position.position_amount))}</span>
                    </div>
                    <div>
                      <span className="text-gray-400">PnL: </span>
                      <span className={parseFloat(position.unrealized_pnl) >= 0 ? "text-green-400" : "text-red-400"}>
                        {parseFloat(position.unrealized_pnl) >= 0 ? "+" : ""}
                        {formatCurrency(parseFloat(position.unrealized_pnl))}
                      </span>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )

  const renderMarginAccount = (account: MarginAccount, exchangeName: string) => (
    <Card className="bg-gradient-to-br from-orange-500/10 to-orange-600/5 border-orange-400/20">
      <CardHeader>
        <CardTitle className="flex items-center space-x-2 text-orange-200">
          <TrendingUp className="w-5 h-5" />
          <span>Margin Trading</span>
          <Badge className={getWalletTypeBadge("margin")}>MARGIN</Badge>
        </CardTitle>
        <CardDescription className="text-gray-300">
          Borrowing and lending for leveraged trades
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Net Value</p>
            <p className="text-lg font-bold text-orange-200">
              {formatCurrency(parseFloat(account.total_net_value))}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-sm text-gray-400">Margin Level</p>
            <p className="text-lg font-bold text-orange-200">
              {account.margin_level ? `${parseFloat(account.margin_level) * 100}%` : "N/A"}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-center h-64">
          <div className="flex items-center space-x-3">
            <RefreshCw className="w-6 h-6 animate-spin text-purple-400" />
            <span className="text-lg text-gray-300">Loading exchange accounts...</span>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <Alert className="border border-red-400/30 bg-red-500/10">
          <AlertTriangle className="h-4 w-4 text-red-400" />
          <AlertTitle className="text-red-300">Error Loading Accounts</AlertTitle>
          <AlertDescription className="text-red-200">
            {error}
          </AlertDescription>
        </Alert>
        <Button onClick={fetchAccounts} className="w-full">
          <RefreshCw className="w-4 h-4 mr-2" />
          Retry
        </Button>
      </div>
    )
  }

  if (accounts.length === 0) {
    return (
      <div className="space-y-6">
        <Alert className="border border-blue-400/30 bg-blue-500/10">
          <AlertTriangle className="h-4 w-4 text-blue-400" />
          <AlertTitle className="text-blue-300">No Exchange Connections</AlertTitle>
          <AlertDescription className="text-blue-200">
            Connect to an exchange first to view your account data.
          </AlertDescription>
        </Alert>
      </div>
    )
  }

  const totals = getTotalValues()

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white">Exchange Accounts</h1>
          <p className="text-gray-400">View all your connected exchange account balances</p>
        </div>
        <Button onClick={fetchAccounts} variant="outline" size="sm">
          <RefreshCw className="w-4 h-4 mr-2" />
          Refresh
        </Button>
      </div>

      {/* Total Portfolio Value */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card className="bg-gradient-to-br from-green-500/10 to-emerald-600/5 border-green-400/20">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2 text-green-200">
              <DollarSign className="w-5 h-5" />
              <span>Total Portfolio Value</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-4xl font-bold text-green-300">
              {formatCurrency(totals.usd)}
            </p>
            <p className="text-lg text-gray-400 mt-2">
              {formatNumber(totals.btc, 8)} BTC
            </p>
          </CardContent>
        </Card>

        <Card className="bg-gradient-to-br from-purple-500/10 to-purple-600/5 border-purple-400/20">
          <CardHeader>
            <CardTitle className="flex items-center space-x-2 text-purple-200">
              <Activity className="w-5 h-5" />
              <span>Connected Exchanges</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-4xl font-bold text-purple-300">
              {accounts.length}
            </p>
            <p className="text-lg text-gray-400 mt-2">
              Active connections
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Exchange Accounts */}
      {accounts.map((accountData) => (
        <Card key={accountData.connection_id} className="bg-gradient-to-br from-white/5 to-white/2 border-white/10">
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center space-x-3">
                <span className="text-2xl">
                  {accountData.exchange_name === 'binance' ? '🔶' : '📊'}
                </span>
                <div>
                  <h3 className="text-xl font-bold text-white">{accountData.display_name}</h3>
                  <p className="text-gray-400 capitalize">{accountData.exchange_name}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="text-2xl font-bold text-white">
                  {formatCurrency(parseFloat(accountData.accounts.total_usd_value))}
                </p>
                <p className="text-sm text-gray-400">
                  Last updated: {new Date(accountData.last_update).toLocaleTimeString()}
                </p>
              </div>
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {accountData.accounts.spot && renderSpotAccount(accountData.accounts.spot, accountData.exchange_name)}
              {accountData.accounts.margin && renderMarginAccount(accountData.accounts.margin, accountData.exchange_name)}
              {accountData.accounts.futures_usdm && renderFuturesAccount(accountData.accounts.futures_usdm, accountData.exchange_name)}
              {accountData.accounts.futures_coinm && renderFuturesAccount(accountData.accounts.futures_coinm, accountData.exchange_name)}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
"use client"

import { useState } from "react"
import { Shield, Plus, AlertTriangle, CheckCircle, ExternalLink, Eye, EyeOff, Info, Lock, Wifi, WifiOff, Settings, Trash2 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { apiClient, type CreateExchangeConnectionRequest, type ExchangeConnection as ExchangeConnectionType } from "@/lib/api"

// Exchange configuration data
const SUPPORTED_EXCHANGES = [
  {
    id: "binance",
    name: "Binance",
    logo: "🔶",
    description: "World's largest crypto exchange",
    apiDocsUrl: "https://binance-docs.github.io/apidocs/spot/en/#general-info",
    setupUrl: "https://www.binance.com/en/my/settings/api-management",
    features: ["Spot Trading", "Futures", "Large Volume", "Low Fees"],
    permissions: ["Read Account", "Spot Trading"],
    securityFeatures: ["IP Whitelist", "2FA Required", "HMAC-SHA256"],
    restrictions: "No withdrawal permissions required",
    status: "available"
  },
  {
    id: "bybit",
    name: "Bybit",
    logo: "🟡",
    description: "Professional derivatives trading",
    apiDocsUrl: "https://bybit-exchange.github.io/docs/v5/intro",
    setupUrl: "https://www.bybit.com/app/user/api-management",
    features: ["Derivatives", "High Leverage", "Fast Execution", "Mobile"],
    permissions: ["Read", "Trade"],
    securityFeatures: ["IP Binding", "OAuth 2.0", "Rate Limiting"],
    restrictions: "Bind to specific server IPs",
    status: "coming_soon"
  },
  {
    id: "coinbase",
    name: "Coinbase Advanced",
    logo: "🔵",
    description: "Trusted US-based exchange",
    apiDocsUrl: "https://docs.cloud.coinbase.com/advanced-trade-api/docs/welcome",
    setupUrl: "https://www.coinbase.com/settings/api",
    features: ["Regulated", "USD Pairs", "Easy Setup", "Insurance"],
    permissions: ["View", "Trade"],
    securityFeatures: ["OAuth Support", "Portfolio Scoped", "2FA Required"],
    restrictions: "Single portfolio per key",
    status: "coming_soon"
  },
  {
    id: "kraken",
    name: "Kraken",
    logo: "🐙",
    description: "Secure European exchange",
    apiDocsUrl: "https://docs.kraken.com/rest/",
    setupUrl: "https://www.kraken.com/u/security/api",
    features: ["High Security", "Regulated", "Advanced Orders", "Low Fees"],
    permissions: ["Query Funds", "Trade"],
    securityFeatures: ["Nonce Protection", "IP Restriction", "QR Code Auth"],
    restrictions: "Private keys never exposed",
    status: "coming_soon"
  },
  {
    id: "kucoin",
    name: "KuCoin",
    logo: "🟢",
    description: "Global crypto exchange with wide altcoin selection",
    apiDocsUrl: "https://docs.kucoin.com/",
    setupUrl: "https://www.kucoin.com/account/api",
    features: ["Wide Selection", "Low Fees", "Futures", "Lending"],
    permissions: ["General", "Trade"],
    securityFeatures: ["Passphrase Required", "IP Whitelist", "2FA Required"],
    restrictions: "Requires API passphrase",
    status: "coming_soon"
  },
  {
    id: "okx",
    name: "OKX",
    logo: "⚫",
    description: "Leading global crypto exchange and Web3 ecosystem",
    apiDocsUrl: "https://www.okx.com/docs-v5/en/",
    setupUrl: "https://www.okx.com/account/my-api",
    features: ["DeFi", "NFTs", "Derivatives", "Copy Trading"],
    permissions: ["Read", "Trade"],
    securityFeatures: ["Passphrase Required", "IP Binding", "OAuth 2.0"],
    restrictions: "Requires API passphrase",
    status: "coming_soon"
  }
]

interface ConnectionFormData {
  apiKey: string
  apiSecret: string
  displayName: string
  password: string
}

interface ExchangeConnectionProps {
  onConnectionSuccess?: () => void
  connections?: ExchangeConnectionType[]
}

export function ExchangeConnection({ onConnectionSuccess, connections = [] }: ExchangeConnectionProps) {
  const [selectedExchange, setSelectedExchange] = useState<string | null>(null)
  const [connectionStatus, setConnectionStatus] = useState<Record<string, 'connected' | 'connecting' | 'error' | 'disconnected'>>({})
  const [showApiSecret, setShowApiSecret] = useState(false)
  const [showPassword, setShowPassword] = useState(false)
  const [formData, setFormData] = useState<ConnectionFormData>({
    apiKey: "",
    apiSecret: "",
    displayName: "",
    password: ""
  })
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState("")

  const handleConnect = async (exchangeId: string) => {
    setConnectionStatus(prev => ({ ...prev, [exchangeId]: 'connecting' }))
    setErrorMessage("")

    try {
      const connectionData: CreateExchangeConnectionRequest = {
        exchange_name: exchangeId,
        display_name: formData.displayName || `${exchangeId} Connection`,
        api_key: formData.apiKey,
        api_secret: formData.apiSecret,
        password: formData.password
      }

      await apiClient.createExchangeConnection(connectionData)

      setConnectionStatus(prev => ({ ...prev, [exchangeId]: 'connected' }))
      onConnectionSuccess?.()
      setIsDialogOpen(false)

      // Clear form data for security
      setFormData({
        apiKey: "",
        apiSecret: "",
        displayName: "",
        password: ""
      })
    } catch (error: any) {
      setConnectionStatus(prev => ({ ...prev, [exchangeId]: 'error' }))
      setErrorMessage(error.message || 'Failed to connect to exchange')
    }
  }

  const handleDisconnect = async (exchangeId: string) => {
    try {
      // Find the connection ID for this exchange
      const connection = connections.find(c => c.exchange_name === exchangeId)
      if (!connection) {
        console.error('Connection not found for exchange:', exchangeId)
        return
      }

      await apiClient.deleteExchangeConnection(connection.id)
      setConnectionStatus(prev => ({ ...prev, [exchangeId]: 'disconnected' }))
      onConnectionSuccess?.() // Refresh the connections list
    } catch (error: any) {
      console.error('Failed to disconnect:', error)
      setErrorMessage(error.message || 'Failed to disconnect from exchange')
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'connected':
        return <CheckCircle className="w-5 h-5 text-emerald-400" />
      case 'connecting':
        return <div className="w-5 h-5 border-2 border-purple-400/20 border-t-purple-400 rounded-full animate-spin" />
      case 'error':
        return <AlertTriangle className="w-5 h-5 text-red-400" />
      default:
        return <WifiOff className="w-5 h-5 text-gray-400" />
    }
  }

  const getStatusText = (status: string) => {
    switch (status) {
      case 'connected': return 'Connected'
      case 'connecting': return 'Connecting...'
      case 'error': return 'Connection Error'
      default: return 'Not Connected'
    }
  }

  return (
    <div className="space-y-8">

      {/* Security Banner */}
      <div className="relative">
        <div className="h-auto w-full border-2 border-[rgba(16,185,129,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.15)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-8 text-white shadow-2xl">
          <div className="flex items-start space-x-6">
            <div className="p-4 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-2xl backdrop-blur-sm border border-emerald-400/20">
              <Shield className="w-8 h-8 text-emerald-200" />
            </div>
            <div className="flex-1 space-y-4">
              <h3 className="text-2xl font-bold bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                Bank-Level Security
              </h3>
              <p className="text-lg text-gray-200 leading-relaxed">
                Your API keys are encrypted and stored securely. We never request withdrawal permissions
                and use industry-standard security practices to protect your funds.
              </p>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mt-6">
                <div className="flex items-center space-x-3 p-3 bg-white/5 rounded-xl backdrop-blur-sm border border-emerald-400/20">
                  <Lock className="w-5 h-5 text-emerald-300" />
                  <span className="font-medium text-emerald-100">End-to-End Encryption</span>
                </div>
                <div className="flex items-center space-x-3 p-3 bg-white/5 rounded-xl backdrop-blur-sm border border-emerald-400/20">
                  <Shield className="w-5 h-5 text-emerald-300" />
                  <span className="font-medium text-emerald-100">Read-Only + Trade</span>
                </div>
                <div className="flex items-center space-x-3 p-3 bg-white/5 rounded-xl backdrop-blur-sm border border-emerald-400/20">
                  <Wifi className="w-5 h-5 text-emerald-300" />
                  <span className="font-medium text-emerald-100">IP Whitelisting</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Exchange Grid */}
      <div className="grid gap-6 md:grid-cols-2">
        {SUPPORTED_EXCHANGES.map((exchange) => {
          const status = connectionStatus[exchange.id] || 'disconnected'
          const isConnected = status === 'connected' || connections.some(c => c.exchange_name === exchange.id && c.is_active)
          const isAvailable = exchange.status === 'available'

          return (
            <div key={exchange.id} className="group">
              <div className="h-auto min-h-[420px] border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(147,51,234,0.4)] hover:from-[rgba(147,51,234,0.15)] hover:to-[rgba(147,51,234,0.05)] hover:translate-y-2 shadow-2xl">
                <div className="space-y-6">
                  {/* Header */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-4">
                      <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-2xl backdrop-blur-sm border border-purple-400/20 group-hover:scale-110 transition-transform duration-300">
                        <div className="text-2xl">{exchange.logo}</div>
                      </div>
                      <div>
                        <h3 className="text-2xl font-bold text-white">{exchange.name}</h3>
                        <p className="text-gray-300 leading-relaxed">
                          {exchange.description}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center space-x-3">
                      {getStatusIcon(isConnected ? 'connected' : status)}
                      <span className="text-sm font-medium text-gray-300">
                        {getStatusText(isConnected ? 'connected' : status)}
                      </span>
                    </div>
                  </div>

                  {/* Features */}
                  <div className="space-y-3">
                    <h4 className="text-lg font-semibold text-purple-200">Features</h4>
                    <div className="flex flex-wrap gap-2">
                      {exchange.features.map((feature) => (
                        <span
                          key={feature}
                          className="px-3 py-1 text-sm bg-gradient-to-r from-purple-500/20 to-pink-500/20 text-purple-200 rounded-xl border border-purple-400/30 backdrop-blur-sm"
                        >
                          {feature}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Security Features */}
                  <div className="space-y-3">
                    <h4 className="text-lg font-semibold text-emerald-200">Security Features</h4>
                    <div className="space-y-2">
                      {exchange.securityFeatures.map((feature) => (
                        <div key={feature} className="flex items-center space-x-3 p-2 bg-white/5 rounded-lg backdrop-blur-sm">
                          <CheckCircle className="w-4 h-4 text-emerald-400" />
                          <span className="font-medium text-gray-200">{feature}</span>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Action Buttons */}
                  <div className="flex gap-3 pt-4">
                    {!isConnected ? (
                      isAvailable ? (
                        <Dialog open={isDialogOpen && selectedExchange === exchange.id} onOpenChange={setIsDialogOpen}>
                          <DialogTrigger asChild>
                            <Button
                              onClick={() => setSelectedExchange(exchange.id)}
                              className="flex-1 h-12 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-semibold text-lg transition-all duration-300 backdrop-blur-sm hover:translate-y-0.5 shadow-lg hover:shadow-xl"
                              disabled={status === 'connecting'}
                            >
                              {status === 'connecting' ? (
                                <>
                                  <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin mr-2" />
                                  Connecting...
                                </>
                              ) : (
                                <>
                                  <Plus className="w-5 h-5 mr-2" />
                                  Connect Securely
                                </>
                              )}
                            </Button>
                          </DialogTrigger>

                          <DialogContent className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl max-w-2xl">
                        <DialogHeader>
                          <DialogTitle className="text-white flex items-center space-x-3">
                            <span className="text-2xl">{exchange.logo}</span>
                            <span>Connect to {exchange.name}</span>
                          </DialogTitle>
                          <DialogDescription className="text-gray-300">
                            Follow these steps to securely connect your {exchange.name} account
                          </DialogDescription>
                        </DialogHeader>

                        <div className="space-y-6">
                          {/* Step-by-step instructions */}
                          <Alert className="border border-amber-400/30 bg-amber-500/10">
                            <Info className="h-4 w-4 text-amber-400" />
                            <AlertTitle className="text-amber-300">Important Security Steps</AlertTitle>
                            <AlertDescription className="text-amber-200">
                              <ol className="list-decimal list-inside space-y-1 mt-2">
                                <li>Only enable "Read Account" and "Trade" permissions</li>
                                <li>Never enable withdrawal or transfer permissions</li>
                                <li>Enable IP whitelisting if available</li>
                                <li>Use 2FA for API key creation</li>
                              </ol>
                            </AlertDescription>
                          </Alert>

                          {/* Quick setup link */}
                          <div className="flex items-center justify-between p-4 border border-purple-400/20 rounded-xl bg-purple-500/5">
                            <div>
                              <h4 className="text-white font-medium">Create API Key</h4>
                              <p className="text-gray-300 text-sm">Go to {exchange.name} to create your API credentials</p>
                            </div>
                            <Button
                              variant="outline"
                              size="sm"
                              className="border-purple-400/50 text-purple-300 hover:text-white hover:bg-purple-500/20"
                              onClick={() => window.open(exchange.setupUrl, '_blank')}
                            >
                              <ExternalLink className="w-4 h-4 mr-2" />
                              Open {exchange.name}
                            </Button>
                          </div>

                          {/* Error Alert */}
                          {errorMessage && (
                            <Alert className="border border-red-400/30 bg-red-500/10">
                              <AlertTriangle className="h-4 w-4 text-red-400" />
                              <AlertDescription className="text-red-200">
                                {errorMessage}
                              </AlertDescription>
                            </Alert>
                          )}

                          {/* Form */}
                          <div className="space-y-4">
                            <div className="space-y-2">
                              <Label htmlFor="displayName" className="text-gray-200 font-medium">Connection Name</Label>
                              <Input
                                id="displayName"
                                type="text"
                                placeholder={`My ${exchange.name} Account`}
                                value={formData.displayName}
                                onChange={(e) => setFormData(prev => ({ ...prev, displayName: e.target.value }))}
                                className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50"
                              />
                            </div>

                            <div className="space-y-2">
                              <Label htmlFor="apiKey" className="text-gray-200 font-medium">API Key</Label>
                              <Input
                                id="apiKey"
                                type="text"
                                placeholder="Enter your API key"
                                value={formData.apiKey}
                                onChange={(e) => setFormData(prev => ({ ...prev, apiKey: e.target.value }))}
                                className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50"
                              />
                            </div>

                            <div className="space-y-2">
                              <Label htmlFor="apiSecret" className="text-gray-200 font-medium">API Secret</Label>
                              <div className="relative">
                                <Input
                                  id="apiSecret"
                                  type={showApiSecret ? "text" : "password"}
                                  placeholder="Enter your API secret"
                                  value={formData.apiSecret}
                                  onChange={(e) => setFormData(prev => ({ ...prev, apiSecret: e.target.value }))}
                                  className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 pr-12"
                                />
                                <button
                                  type="button"
                                  onClick={() => setShowApiSecret(!showApiSecret)}
                                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors"
                                >
                                  {showApiSecret ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                                </button>
                              </div>
                            </div>

                            <div className="space-y-2">
                              <Label htmlFor="password" className="text-gray-200 font-medium">Account Password</Label>
                              <div className="relative">
                                <Input
                                  id="password"
                                  type={showPassword ? "text" : "password"}
                                  placeholder="Enter your account password for encryption"
                                  value={formData.password}
                                  onChange={(e) => setFormData(prev => ({ ...prev, password: e.target.value }))}
                                  className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 pr-12"
                                />
                                <button
                                  type="button"
                                  onClick={() => setShowPassword(!showPassword)}
                                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors"
                                >
                                  {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                                </button>
                              </div>
                              <p className="text-xs text-gray-400">
                                Used to encrypt your API keys. We never store this password.
                              </p>
                            </div>
                          </div>

                          {/* Submit */}
                          <div className="flex gap-3">
                            <Button
                              onClick={() => setIsDialogOpen(false)}
                              variant="outline"
                              className="flex-1 border-white/20 text-gray-300 hover:text-white hover:bg-white/10"
                            >
                              Cancel
                            </Button>
                            <Button
                              onClick={() => handleConnect(exchange.id)}
                              disabled={!formData.apiKey || !formData.apiSecret || !formData.password || connectionStatus[exchange.id] === 'connecting'}
                              className="flex-1 h-12 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-medium transition-all duration-300 disabled:opacity-50"
                            >
                              {connectionStatus[exchange.id] === 'connecting' ? (
                                <>
                                  <div className="w-4 h-4 border-2 border-white/20 border-t-white rounded-full animate-spin mr-2" />
                                  Connecting...
                                </>
                              ) : (
                                <>
                                  <Shield className="w-4 h-4 mr-2" />
                                  Connect Securely
                                </>
                              )}
                            </Button>
                          </div>
                        </div>
                          </DialogContent>
                        </Dialog>
                      ) : (
                        <Button
                          disabled
                          className="flex-1 h-12 border border-gray-500/50 rounded-xl bg-gray-600/20 text-gray-400 font-semibold text-lg transition-all duration-300 backdrop-blur-sm cursor-not-allowed"
                        >
                          <Plus className="w-5 h-5 mr-2" />
                          Coming Soon
                        </Button>
                      )
                    ) : (
                      <div className="flex gap-3">
                        <Button
                          className="flex-1 h-10 border border-white/20 rounded-xl bg-white/10 hover:bg-white/20 text-gray-200 hover:text-white font-medium transition-all duration-300 backdrop-blur-sm"
                        >
                          <Settings className="w-4 h-4 mr-2" />
                          Manage
                        </Button>
                        <Button
                          onClick={() => handleDisconnect(exchange.id)}
                          className="flex-1 h-10 border border-red-400/50 rounded-xl bg-red-600/20 hover:bg-red-500/30 text-red-300 hover:text-white font-medium transition-all duration-300 backdrop-blur-sm"
                        >
                          <Trash2 className="w-4 h-4 mr-2" />
                          Disconnect
                        </Button>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          )
        })}
      </div>

      {/* Security Information */}
      <div className="relative">
        <div className="h-auto w-full border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8 text-white shadow-2xl">
          <div className="space-y-6">
            <div className="text-center space-y-2">
              <h3 className="text-2xl font-bold bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Security & Privacy
              </h3>
              <p className="text-lg text-gray-300">
                Your trust is our priority. Here's how we protect your trading credentials.
              </p>
            </div>

            <div className="grid gap-8 md:grid-cols-2">
              <div className="space-y-4">
                <div className="flex items-center space-x-3">
                  <div className="p-2 bg-emerald-500/20 rounded-lg">
                    <CheckCircle className="w-5 h-5 text-emerald-400" />
                  </div>
                  <h4 className="text-xl font-semibold text-emerald-200">What we do</h4>
                </div>
                <div className="space-y-3">
                  {[
                    "Encrypt all API keys with AES-256",
                    "Store keys on secure, isolated servers",
                    "Use read-only and trade-only permissions",
                    "Validate connections before storing"
                  ].map((item) => (
                    <div key={item} className="flex items-center space-x-3 p-3 bg-white/5 rounded-xl backdrop-blur-sm">
                      <CheckCircle className="w-4 h-4 text-emerald-400 flex-shrink-0" />
                      <span className="font-medium text-gray-200">{item}</span>
                    </div>
                  ))}
                </div>
              </div>

              <div className="space-y-4">
                <div className="flex items-center space-x-3">
                  <div className="p-2 bg-red-500/20 rounded-lg">
                    <AlertTriangle className="w-5 h-5 text-red-400" />
                  </div>
                  <h4 className="text-xl font-semibold text-red-200">What we never do</h4>
                </div>
                <div className="space-y-3">
                  {[
                    "Request withdrawal permissions",
                    "Share your keys with third parties",
                    "Store keys in browser or frontend",
                    "Access your funds directly"
                  ].map((item) => (
                    <div key={item} className="flex items-center space-x-3 p-3 bg-white/5 rounded-xl backdrop-blur-sm">
                      <AlertTriangle className="w-4 h-4 text-red-400 flex-shrink-0" />
                      <span className="font-medium text-gray-200">{item}</span>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
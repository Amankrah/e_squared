"use client"

import { useState, useEffect } from "react"
import { MoreHorizontal, Settings, Trash2, RefreshCw, Eye, Shield, AlertTriangle, Activity, Calendar, DollarSign } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { apiClient, type ExchangeConnection as ApiExchangeConnection } from "@/lib/api"
import { useAuth } from "@/contexts/auth-context"

interface ExchangeConnection {
  id: string
  exchange: string
  exchangeName: string
  logo: string
  status: 'active' | 'inactive' | 'error' | 'testing'
  connectedAt: string
  lastActivity: string
  permissions: string[]
  ipWhitelist?: string[]
  balance?: {
    total: string
    available: string
    currency: string
  }
  performance?: {
    totalTrades: number
    successRate: number
    profit: string
  }
  liveBalanceData?: any
}

interface ExchangeConnectionManagerProps {
  onConnectionUpdate?: () => void
}

export function ExchangeConnectionManager({
  onConnectionUpdate
}: ExchangeConnectionManagerProps) {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [connections, setConnections] = useState<ExchangeConnection[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [selectedConnection, setSelectedConnection] = useState<ExchangeConnection | null>(null)
  const [isDetailsOpen, setIsDetailsOpen] = useState(false)
  const [isDisconnectOpen, setIsDisconnectOpen] = useState(false)
  const [isSyncing, setIsSyncing] = useState<string | null>(null)
  const [isLoadingLiveBalances, setIsLoadingLiveBalances] = useState(false)

  // Convert API connection to UI connection format with live balance data
  const mapApiConnectionToUi = (apiConnection: ApiExchangeConnection, liveBalanceData?: any): ExchangeConnection => {
    const exchangeLogos: Record<string, string> = {
      'binance': '🟡',
      'coinbase': '🔵',
      'kraken': '🟣'
    }

    // Extract total USD value from live balance data
    let totalBalance = 0
    if (liveBalanceData && liveBalanceData.total_usd_value) {
      totalBalance = parseFloat(liveBalanceData.total_usd_value) || 0
    }

    const status = apiConnection.connection_status === 'connected' ? 'active' :
                   apiConnection.connection_status === 'error' ? 'error' : 'inactive'

    return {
      id: apiConnection.id,
      exchange: apiConnection.exchange_name,
      exchangeName: apiConnection.display_name,
      logo: exchangeLogos[apiConnection.exchange_name.toLowerCase()] || '🔗',
      status: status as 'active' | 'inactive' | 'error' | 'testing',
      connectedAt: apiConnection.created_at,
      lastActivity: apiConnection.last_sync || apiConnection.updated_at,
      permissions: ['Read', 'Trade'], // Default permissions - could be expanded
      balance: totalBalance > 0 ? {
        total: totalBalance.toFixed(2),
        available: totalBalance.toFixed(2),
        currency: 'USD'
      } : undefined,
      // Store the raw live balance data for detailed view
      liveBalanceData: liveBalanceData || undefined
    }
  }

  // SECURE: Only load connections when authenticated
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadConnections()
    } else if (!authLoading && !isAuthenticated) {
      // User is not authenticated, clear any existing connections
      setConnections([])
      setIsLoading(false)
    }
  }, [isAuthenticated, authLoading])

  const loadConnections = async (password?: string) => {
    // SECURE: Double-check authentication before API call
    if (!isAuthenticated) {
      console.warn('Attempted to load connections without authentication')
      return
    }

    try {
      setIsLoading(true)
      const response = await apiClient.getExchangeConnections()

      // If password is provided, try to load live balances
      if (password) {
        try {
          const liveBalancesResponse = await apiClient.getAllLiveUserBalances(password)

          // Create a map of connection_id to live balance data
          const liveBalancesMap: Record<string, any> = {}
          if (liveBalancesResponse.balances) {
            for (const balance of liveBalancesResponse.balances) {
              liveBalancesMap[balance.exchange_connection_id] = balance
            }
          }

          const connectionsWithLiveBalances = response.connections.map(connection =>
            mapApiConnectionToUi(connection, liveBalancesMap[connection.id])
          )

          setConnections(connectionsWithLiveBalances)
        } catch (liveBalanceError) {
          console.error('Failed to load live balances, falling back to connections only:', liveBalanceError)
          // Fall back to just showing connections without balance data
          const connectionsOnly = response.connections.map(connection =>
            mapApiConnectionToUi(connection)
          )
          setConnections(connectionsOnly)
        }
      } else {
        // No password provided, just show connections without live balance data
        const connectionsOnly = response.connections.map(connection =>
          mapApiConnectionToUi(connection)
        )
        setConnections(connectionsOnly)
      }
    } catch (error) {
      console.error('Failed to load exchange connections:', error)
      // SECURE: If 401 error, the auth context will handle it
    } finally {
      setIsLoading(false)
    }
  }

  const handleReconnect = async (connectionId: string) => {
    // SECURE: Verify authentication before action
    if (!isAuthenticated) {
      console.warn('Attempted to sync balances without authentication')
      return
    }

    // Prompt for password to decrypt API credentials
    const password = prompt('Enter your account password to sync balances:')
    if (!password) {
      return
    }

    try {
      setIsSyncing(connectionId)
      const syncResponse = await apiClient.syncExchangeConnection(connectionId, password)
      console.log('Sync response:', syncResponse)

      // Reload connections with live balance data using the same password
      await loadConnections(password)
      onConnectionUpdate?.()
      alert('Connection synced successfully! Live balance data updated.')
    } catch (error) {
      console.error('Failed to sync balances:', error)
      alert('Failed to sync balances. Please check your password and try again.')
    } finally {
      setIsSyncing(null)
    }
  }

  const handleDisconnect = async (connection: ExchangeConnection) => {
    // SECURE: Verify authentication before action
    if (!isAuthenticated) {
      console.warn('Attempted to disconnect exchange without authentication')
      return
    }

    try {
      await apiClient.deleteExchangeConnection(connection.id)
      await loadConnections() // Refresh connections
      onConnectionUpdate?.()
      setIsDisconnectOpen(false)
      setSelectedConnection(null)
    } catch (error) {
      console.error('Failed to disconnect exchange:', error)
    }
  }

  const handleLoadLiveBalances = async () => {
    // SECURE: Verify authentication before action
    if (!isAuthenticated) {
      console.warn('Attempted to load live balances without authentication')
      return
    }

    // Prompt for password to decrypt API credentials
    const password = prompt('Enter your account password to load live balances for all exchanges:')
    if (!password) {
      return
    }

    try {
      setIsLoadingLiveBalances(true)
      await loadConnections(password)
      onConnectionUpdate?.()
    } catch (error) {
      console.error('Failed to load live balances:', error)
      alert('Failed to load live balances. Please check your password and try again.')
    } finally {
      setIsLoadingLiveBalances(false)
    }
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active': return 'text-emerald-400 bg-emerald-500/20'
      case 'inactive': return 'text-gray-400 bg-gray-500/20'
      case 'error': return 'text-red-400 bg-red-500/20'
      case 'testing': return 'text-amber-400 bg-amber-500/20'
      default: return 'text-gray-400 bg-gray-500/20'
    }
  }

  const getStatusText = (status: string) => {
    switch (status) {
      case 'active': return 'Active'
      case 'inactive': return 'Inactive'
      case 'error': return 'Error'
      case 'testing': return 'Testing'
      default: return 'Unknown'
    }
  }

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  }

  // SECURE: Show loading state while auth is being checked
  if (authLoading) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 text-center">
        <div className="max-w-md mx-auto space-y-4">
          <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
            <RefreshCw className="w-8 h-8 text-purple-300 animate-spin" />
          </div>
          <h3 className="text-xl font-semibold text-white">Checking Authentication...</h3>
          <p className="text-gray-300 leading-relaxed">
            Verifying your credentials for secure access.
          </p>
        </div>
      </div>
    )
  }

  // SECURE: Show login prompt if not authenticated
  if (!isAuthenticated) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 text-center">
        <div className="max-w-md mx-auto space-y-4">
          <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
            <Shield className="w-8 h-8 text-purple-300" />
          </div>
          <h3 className="text-xl font-semibold text-white">Authentication Required</h3>
          <p className="text-gray-300 leading-relaxed">
            Please log in to view and manage your exchange connections.
            Your connections are secured with bank-level encryption.
          </p>
        </div>
      </div>
    )
  }

  if (isLoading) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 text-center">
        <div className="max-w-md mx-auto space-y-4">
          <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
            <RefreshCw className="w-8 h-8 text-purple-300 animate-spin" />
          </div>
          <h3 className="text-xl font-semibold text-white">Loading Connections...</h3>
          <p className="text-gray-300 leading-relaxed">
            Fetching your exchange connections and balances.
          </p>
        </div>
      </div>
    )
  }

  if (connections.length === 0) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 text-center">
        <div className="max-w-md mx-auto space-y-4">
          <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
            <Shield className="w-8 h-8 text-purple-300" />
          </div>
          <h3 className="text-xl font-semibold text-white">No Exchange Connections</h3>
          <p className="text-gray-300 leading-relaxed">
            Connect your first exchange to start building and executing trading strategies.
            Your connections are secured with bank-level encryption.
          </p>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header with Load Live Balances Button */}
      {connections.length > 0 && (
        <div className="flex justify-between items-center">
          <div>
            <h2 className="text-xl font-semibold text-white">Exchange Connections</h2>
            <p className="text-sm text-gray-400">Manage your exchange connections and view live balances</p>
          </div>
          <Button
            onClick={handleLoadLiveBalances}
            disabled={isLoadingLiveBalances}
            className="h-10 border border-emerald-400/50 rounded-xl bg-gradient-to-r from-emerald-600/90 to-teal-600/90 hover:from-emerald-500/95 hover:to-teal-500/95 text-white font-medium transition-all duration-300 disabled:opacity-50"
          >
            <DollarSign className={`w-4 h-4 mr-2 ${isLoadingLiveBalances ? 'animate-pulse' : ''}`} />
            {isLoadingLiveBalances ? 'Loading Live Balances...' : 'Load Live Balances'}
          </Button>
        </div>
      )}

      {/* Connections Grid */}
      <div className="grid gap-6">
        {connections.map((connection) => (
          <Card key={connection.id} className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
            <CardHeader className="pb-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-4">
                  <div className="text-2xl">{connection.logo}</div>
                  <div>
                    <CardTitle className="text-white">{connection.exchangeName}</CardTitle>
                    <CardDescription className="text-gray-300">
                      Connected {formatDate(connection.connectedAt)}
                    </CardDescription>
                  </div>
                </div>

                <div className="flex items-center space-x-3">
                  <span className={`px-3 py-1 rounded-full text-xs font-medium ${getStatusColor(connection.status)}`}>
                    {getStatusText(connection.status)}
                  </span>

                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 rounded-lg text-gray-400 hover:text-white hover:bg-white/10"
                      >
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent className="border-2 border-[rgba(147,51,234,0.3)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                      <DropdownMenuItem
                        onClick={() => {
                          setSelectedConnection(connection)
                          setIsDetailsOpen(true)
                        }}
                        className="text-gray-200 hover:text-white hover:bg-white/10 focus:bg-white/10 cursor-pointer"
                      >
                        <Eye className="w-4 h-4 mr-2" />
                        View Details
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => handleReconnect(connection.id)}
                        disabled={isSyncing === connection.id}
                        className="text-gray-200 hover:text-white hover:bg-white/10 focus:bg-white/10 cursor-pointer disabled:opacity-50"
                      >
                        <RefreshCw className={`w-4 h-4 mr-2 ${isSyncing === connection.id ? 'animate-spin' : ''}`} />
                        {isSyncing === connection.id ? 'Syncing...' : 'Sync Balances'}
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => {
                          setSelectedConnection(connection)
                          setIsDisconnectOpen(true)
                        }}
                        className="text-red-300 hover:text-white hover:bg-red-500/20 focus:bg-red-500/20 cursor-pointer"
                      >
                        <Trash2 className="w-4 h-4 mr-2" />
                        Disconnect
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </div>
              </div>
            </CardHeader>

            <CardContent className="space-y-4">
              {/* Quick Stats */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div className="text-center p-3 bg-white/5 rounded-xl">
                  <DollarSign className="w-4 h-4 text-emerald-400 mx-auto mb-1" />
                  <p className="text-xs text-gray-400">Balance</p>
                  <p className="text-sm font-medium text-white">
                    {connection.balance ? `${connection.balance.available} ${connection.balance.currency}` : 'N/A'}
                  </p>
                </div>
                <div className="text-center p-3 bg-white/5 rounded-xl">
                  <Activity className="w-4 h-4 text-purple-400 mx-auto mb-1" />
                  <p className="text-xs text-gray-400">Trades</p>
                  <p className="text-sm font-medium text-white">
                    {connection.performance?.totalTrades || 0}
                  </p>
                </div>
                <div className="text-center p-3 bg-white/5 rounded-xl">
                  <Calendar className="w-4 h-4 text-blue-400 mx-auto mb-1" />
                  <p className="text-xs text-gray-400">Last Active</p>
                  <p className="text-sm font-medium text-white">
                    {formatDate(connection.lastActivity).split(',')[0]}
                  </p>
                </div>
                <div className="text-center p-3 bg-white/5 rounded-xl">
                  <Shield className="w-4 h-4 text-emerald-400 mx-auto mb-1" />
                  <p className="text-xs text-gray-400">Security</p>
                  <p className="text-sm font-medium text-white">
                    {connection.ipWhitelist ? 'IP Restricted' : 'Standard'}
                  </p>
                </div>
              </div>

              {/* Permissions */}
              <div className="flex flex-wrap gap-2">
                {connection.permissions.map((permission) => (
                  <span
                    key={permission}
                    className="px-2 py-1 text-xs bg-emerald-500/20 text-emerald-200 rounded-lg border border-emerald-400/30"
                  >
                    {permission}
                  </span>
                ))}
              </div>

              {/* Status Alert */}
              {connection.status === 'error' && (
                <Alert className="border border-red-400/30 bg-red-500/10">
                  <AlertTriangle className="h-4 w-4 text-red-400" />
                  <AlertDescription className="text-red-200">
                    Connection error detected. Please check your API credentials and reconnect.
                  </AlertDescription>
                </Alert>
              )}
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Connection Details Modal */}
      <Dialog open={isDetailsOpen} onOpenChange={setIsDetailsOpen}>
        <DialogContent className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl max-w-2xl">
          {selectedConnection && (
            <>
              <DialogHeader>
                <DialogTitle className="text-white flex items-center space-x-3">
                  <span className="text-2xl">{selectedConnection.logo}</span>
                  <span>{selectedConnection.exchangeName} Connection Details</span>
                </DialogTitle>
                <DialogDescription className="text-gray-300">
                  Detailed information about your exchange connection
                </DialogDescription>
              </DialogHeader>

              <div className="space-y-6">
                {/* Connection Info */}
                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-3">
                    <h4 className="text-purple-200 font-medium">Connection Details</h4>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-400">Status:</span>
                        <span className={`px-2 py-1 rounded text-xs ${getStatusColor(selectedConnection.status)}`}>
                          {getStatusText(selectedConnection.status)}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Connected:</span>
                        <span className="text-white">{formatDate(selectedConnection.connectedAt)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Last Activity:</span>
                        <span className="text-white">{formatDate(selectedConnection.lastActivity)}</span>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-3">
                    <h4 className="text-purple-200 font-medium">Security</h4>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-400">Encryption:</span>
                        <span className="text-emerald-300">AES-256</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">IP Whitelist:</span>
                        <span className="text-white">
                          {selectedConnection.ipWhitelist ? 'Enabled' : 'Disabled'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-400">Key Rotation:</span>
                        <span className="text-white">Manual</span>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Permissions */}
                <div className="space-y-3">
                  <h4 className="text-purple-200 font-medium">API Permissions</h4>
                  <div className="flex flex-wrap gap-2">
                    {selectedConnection.permissions.map((permission) => (
                      <span
                        key={permission}
                        className="px-3 py-1 text-sm bg-emerald-500/20 text-emerald-200 rounded-lg border border-emerald-400/30"
                      >
                        {permission}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Performance */}
                {selectedConnection.performance && (
                  <div className="space-y-3">
                    <h4 className="text-purple-200 font-medium">Trading Performance</h4>
                    <div className="grid gap-4 md:grid-cols-3">
                      <div className="text-center p-4 bg-white/5 rounded-xl">
                        <p className="text-2xl font-bold text-white">{selectedConnection.performance.totalTrades}</p>
                        <p className="text-sm text-gray-400">Total Trades</p>
                      </div>
                      <div className="text-center p-4 bg-white/5 rounded-xl">
                        <p className="text-2xl font-bold text-emerald-400">{selectedConnection.performance.successRate}%</p>
                        <p className="text-sm text-gray-400">Success Rate</p>
                      </div>
                      <div className="text-center p-4 bg-white/5 rounded-xl">
                        <p className="text-2xl font-bold text-purple-400">{selectedConnection.performance.profit}</p>
                        <p className="text-sm text-gray-400">Total Profit</p>
                      </div>
                    </div>
                  </div>
                )}

                {/* Actions */}
                <div className="flex gap-3">
                  <Button
                    onClick={() => setIsDetailsOpen(false)}
                    variant="outline"
                    className="flex-1 border-white/20 text-gray-300 hover:text-white hover:bg-white/10"
                  >
                    Close
                  </Button>
                  <Button
                    onClick={() => handleReconnect(selectedConnection.id)}
                    disabled={isSyncing === selectedConnection.id}
                    className="flex-1 h-10 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-medium transition-all duration-300 disabled:opacity-50"
                  >
                    <RefreshCw className={`w-4 h-4 mr-2 ${isSyncing === selectedConnection.id ? 'animate-spin' : ''}`} />
                    {isSyncing === selectedConnection.id ? 'Syncing...' : 'Sync Balances'}
                  </Button>
                </div>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>

      {/* Disconnect Confirmation Modal */}
      <Dialog open={isDisconnectOpen} onOpenChange={setIsDisconnectOpen}>
        <DialogContent className="border-2 border-[rgba(244,63,94,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(244,63,94,0.15)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl">
          {selectedConnection && (
            <>
              <DialogHeader>
                <DialogTitle className="text-white flex items-center space-x-3">
                  <AlertTriangle className="w-6 h-6 text-red-400" />
                  <span>Disconnect {selectedConnection.exchangeName}?</span>
                </DialogTitle>
                <DialogDescription className="text-gray-300">
                  This will permanently remove your connection to {selectedConnection.exchangeName}.
                  Any active strategies using this connection will be paused.
                </DialogDescription>
              </DialogHeader>

              <Alert className="border border-red-400/30 bg-red-500/10">
                <AlertTriangle className="h-4 w-4 text-red-400" />
                <AlertTitle className="text-red-300">Warning</AlertTitle>
                <AlertDescription className="text-red-200">
                  This action cannot be undone. You'll need to reconnect and reconfigure
                  your API keys to resume trading.
                </AlertDescription>
              </Alert>

              <div className="flex gap-3">
                <Button
                  onClick={() => setIsDisconnectOpen(false)}
                  variant="outline"
                  className="flex-1 border-white/20 text-gray-300 hover:text-white hover:bg-white/10"
                >
                  Cancel
                </Button>
                <Button
                  onClick={() => handleDisconnect(selectedConnection)}
                  className="flex-1 h-10 border border-red-400/50 rounded-xl bg-gradient-to-r from-red-600/90 to-red-700/90 hover:from-red-500/95 hover:to-red-600/95 text-white font-medium transition-all duration-300"
                >
                  <Trash2 className="w-4 h-4 mr-2" />
                  Disconnect
                </Button>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
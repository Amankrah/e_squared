"use client"

import { useState, useEffect } from "react"
import { Plus, Shield, Activity, Settings } from "lucide-react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { ExchangeConnection } from "@/components/dashboard/exchange-connection"
import { ExchangeConnectionManager } from "@/components/dashboard/exchange-connection-manager"
import { ExchangeConnectionTest } from "@/components/dashboard/exchange-connection-test"
import { useAuth } from "@/contexts/auth-context"
import { apiClient, type ExchangeConnection as ExchangeConnectionType } from "@/lib/api"

export default function ExchangesPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [connections, setConnections] = useState<ExchangeConnectionType[]>([])
  const [selectedTab, setSelectedTab] = useState("connections")
  const [loading, setLoading] = useState(false)
  const [stats, setStats] = useState({
    totalTrades: 0,
    avgSuccessRate: 0,
    supportedExchanges: 6
  })

  // SECURE: Only load data when authenticated
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadConnections()
    } else if (!authLoading && !isAuthenticated) {
      setConnections([])
      setLoading(false)
    }
  }, [isAuthenticated, authLoading])

  const loadConnections = async () => {
    // SECURE: Double-check authentication before API calls
    if (!isAuthenticated) {
      console.warn('Attempted to load connections without authentication')
      return
    }

    try {
      setLoading(true)
      const response = await apiClient.getExchangeConnections()
      setConnections(response.connections || [])

      // Calculate real stats from actual connections
      const totalTrades = response.connections?.reduce((sum, conn) => {
        // Note: Backend doesn't currently provide trade stats, so this will be 0
        return sum + 0 // TODO: Add trade statistics to backend
      }, 0) || 0

      const avgSuccessRate = response.connections?.length > 0
        ? 0 // TODO: Calculate from actual trade data when available
        : 0

      setStats({
        totalTrades,
        avgSuccessRate,
        supportedExchanges: 6 // This is the actual number of supported exchanges
      })
    } catch (error) {
      console.error('Failed to load connections:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleConnectionSuccess = () => {
    // Refresh connections from backend
    loadConnections()
  }

  const handleReconnect = async (connectionId: string) => {
    try {
      // TODO: Implement reconnection API when backend supports it
      console.log(`Reconnecting connection: ${connectionId}`)
      // For now, just refresh the connections list
      await loadConnections()
    } catch (error) {
      console.error('Failed to reconnect:', error)
    }
  }

  const handleDisconnect = async (connectionId: string) => {
    try {
      await apiClient.deleteExchangeConnection(connectionId)
      // Refresh connections list after successful deletion
      await loadConnections()
    } catch (error) {
      console.error('Failed to disconnect:', error)
    }
  }

  // SECURE: Show loading state while auth is being checked
  if (authLoading) {
    return (
      <DashboardLayout>
        <div className="flex items-center justify-center min-h-[400px]">
          <div className="text-center space-y-4">
            <div className="p-4 bg-purple-500/20 rounded-2xl w-fit mx-auto">
              <Activity className="w-8 h-8 text-purple-300 animate-spin" />
            </div>
            <h3 className="text-xl font-semibold text-white">Loading Exchange Connections...</h3>
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
            <p className="text-gray-300">Please log in to access your exchange connections and manage your trading accounts.</p>
            <a href="/login">
              <button className="bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 px-6 py-2 rounded-lg text-white font-medium">
                Go to Login
              </button>
            </a>
          </div>
        </div>
      </DashboardLayout>
    )
  }

  return (
    <DashboardLayout>
      <div className="space-y-8">
        {/* Compact Page Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Exchange Connections
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Securely connect and manage your cryptocurrency exchange accounts
            </p>
          </div>

          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2 px-3 py-2 bg-emerald-500/10 rounded-lg border border-emerald-400/20 backdrop-blur-sm">
              <Shield className="w-4 h-4 text-emerald-400" />
              <span className="text-sm font-medium text-emerald-200">
                {loading ? '...' : connections.filter(c => c.connection_status === 'connected').length} Active
              </span>
            </div>
          </div>
        </div>

        {/* Real Quick Stats */}
        <div className="grid gap-4 md:grid-cols-4 mb-6">
          <div className="border border-[rgba(16,185,129,0.2)] rounded-xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-emerald-500/20 rounded-lg">
                <Shield className="w-5 h-5 text-emerald-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Active</p>
                <p className="text-xl font-bold text-white">
                  {loading ? '...' : connections.filter(c => c.connection_status === 'connected').length}
                </p>
              </div>
            </div>
          </div>

          <div className="border border-[rgba(147,51,234,0.2)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-purple-500/20 rounded-lg">
                <Activity className="w-5 h-5 text-purple-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Total Connections</p>
                <p className="text-xl font-bold text-white">
                  {loading ? '...' : connections.length}
                </p>
              </div>
            </div>
          </div>

          <div className="border border-[rgba(59,130,246,0.2)] rounded-xl bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-blue-500/20 rounded-lg">
                <Settings className="w-5 h-5 text-blue-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Connected</p>
                <p className="text-xl font-bold text-white">
                  {loading ? '...' : `${connections.length > 0 ? Math.round((connections.filter(c => c.connection_status === 'connected').length / connections.length) * 100) : 0}%`}
                </p>
              </div>
            </div>
          </div>

          <div className="border border-[rgba(244,63,94,0.2)] rounded-xl bg-gradient-to-br from-[rgba(244,63,94,0.1)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-pink-500/20 rounded-lg">
                <Plus className="w-5 h-5 text-pink-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Supported</p>
                <p className="text-xl font-bold text-white">{stats.supportedExchanges}</p>
              </div>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <Tabs value={selectedTab} onValueChange={setSelectedTab} className="space-y-6">
          <TabsList className="grid grid-cols-3 w-fit h-12 rounded-xl border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-1">
            <TabsTrigger
              value="connections"
              className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium transition-all duration-300 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 hover:shadow-lg cursor-pointer active:scale-95"
            >
              <Shield className="w-4 h-4 mr-2 transition-transform group-hover:rotate-12" />
              My Connections
            </TabsTrigger>
            <TabsTrigger
              value="add"
              className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium transition-all duration-300 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 hover:shadow-lg cursor-pointer active:scale-95"
            >
              <Plus className="w-4 h-4 mr-2 transition-transform group-hover:rotate-90" />
              Add Exchange
            </TabsTrigger>
            <TabsTrigger
              value="test"
              className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium transition-all duration-300 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 hover:shadow-lg cursor-pointer active:scale-95"
            >
              <Activity className="w-4 h-4 mr-2 transition-transform group-hover:scale-110" />
              Test Connection
            </TabsTrigger>
          </TabsList>

          <TabsContent value="connections" className="space-y-6">
            <ExchangeConnectionManager />
          </TabsContent>

          <TabsContent value="add" className="space-y-6">
            <ExchangeConnection
              onConnectionSuccess={handleConnectionSuccess}
              connections={connections}
            />
          </TabsContent>

          <TabsContent value="test" className="space-y-6">
            <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8">
              <ExchangeConnectionTest
                exchange="binance"
                onTestComplete={(success, results) => {
                  console.log('Test completed:', { success, results })
                }}
              />
            </div>
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
}
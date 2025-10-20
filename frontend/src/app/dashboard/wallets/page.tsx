"use client"

import { useState, useEffect } from "react"
import { Plus, Wallet, Activity, Shield } from "lucide-react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { WalletConnectionManager } from "@/components/dashboard/wallet-connection-manager"
import { AddWalletDialog } from "@/components/dashboard/add-wallet-dialog"
import { useAuth } from "@/contexts/auth-context"
import { apiClient, type WalletConnection } from "@/lib/api"

export default function WalletsPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth()
  const [wallets, setWallets] = useState<WalletConnection[]>([])
  const [selectedTab, setSelectedTab] = useState("wallets")
  const [loading, setLoading] = useState(false)
  const [stats, setStats] = useState({
    totalWallets: 0,
    activeWallets: 0,
    supportedBlockchains: 3  // Ethereum, BNB Chain, Solana
  })

  // SECURE: Only load data when authenticated
  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      loadWallets()
    } else if (!authLoading && !isAuthenticated) {
      setWallets([])
      setLoading(false)
    }
  }, [isAuthenticated, authLoading])

  const loadWallets = async () => {
    // SECURE: Double-check authentication before API calls
    if (!isAuthenticated) {
      console.warn('Attempted to load wallets without authentication')
      return
    }

    try {
      setLoading(true)
      const response = await apiClient.getWalletConnections()
      setWallets(response.wallets || [])

      // Calculate stats from actual wallet data
      setStats({
        totalWallets: response.wallets?.length || 0,
        activeWallets: response.wallets?.filter(w => w.is_active && w.connection_status === 'connected').length || 0,
        supportedBlockchains: 3
      })
    } catch (error) {
      console.error('Failed to load wallets:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleWalletAdded = () => {
    // Refresh wallets from backend
    loadWallets()
  }

  const handleDisconnect = async (walletId: string) => {
    try {
      await apiClient.deleteWalletConnection(walletId)
      // Refresh wallets list after successful deletion
      await loadWallets()
    } catch (error) {
      console.error('Failed to disconnect wallet:', error)
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
            <h3 className="text-xl font-semibold text-white">Loading Wallet Connections...</h3>
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
            <p className="text-gray-300">Please log in to access your wallet connections and manage DEX trading.</p>
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
        {/* Page Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 mb-6">
          <div>
            <h1 className="text-2xl lg:text-3xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Crypto Wallets
              </span>
            </h1>
            <p className="text-gray-300 mt-1">
              Connect your crypto wallets for decentralized exchange (DEX) trading
            </p>
          </div>

          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2 px-3 py-2 bg-emerald-500/10 rounded-lg border border-emerald-400/20 backdrop-blur-sm">
              <Wallet className="w-4 h-4 text-emerald-400" />
              <span className="text-sm font-medium text-emerald-200">
                {loading ? '...' : stats.activeWallets} Active
              </span>
            </div>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid gap-4 md:grid-cols-3 mb-6">
          <div className="border border-[rgba(16,185,129,0.2)] rounded-xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-emerald-500/20 rounded-lg">
                <Wallet className="w-5 h-5 text-emerald-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Active Wallets</p>
                <p className="text-xl font-bold text-white">
                  {loading ? '...' : stats.activeWallets}
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
                <p className="text-xs text-gray-400 uppercase tracking-wide">Total Wallets</p>
                <p className="text-xl font-bold text-white">
                  {loading ? '...' : stats.totalWallets}
                </p>
              </div>
            </div>
          </div>

          <div className="border border-[rgba(59,130,246,0.2)] rounded-xl bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl p-4 text-white">
            <div className="flex items-center space-x-3">
              <div className="p-2 bg-blue-500/20 rounded-lg">
                <Shield className="w-5 h-5 text-blue-300" />
              </div>
              <div>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Blockchains</p>
                <p className="text-xl font-bold text-white">{stats.supportedBlockchains}</p>
              </div>
            </div>
          </div>
        </div>

        {/* Main Content */}
        <Tabs value={selectedTab} onValueChange={setSelectedTab} className="space-y-6">
          <TabsList className="grid grid-cols-2 w-fit h-12 rounded-xl border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-1">
            <TabsTrigger
              value="wallets"
              className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium transition-all duration-300 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 hover:shadow-lg cursor-pointer active:scale-95"
            >
              <Wallet className="w-4 h-4 mr-2 transition-transform group-hover:rotate-12" />
              My Wallets
            </TabsTrigger>
            <TabsTrigger
              value="add"
              className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-purple-600/90 data-[state=active]:to-pink-600/90 data-[state=active]:text-white text-gray-300 font-medium transition-all duration-300 hover:bg-gradient-to-r hover:from-purple-500/20 hover:to-pink-500/20 hover:text-white hover:scale-105 hover:shadow-lg cursor-pointer active:scale-95"
            >
              <Plus className="w-4 h-4 mr-2 transition-transform group-hover:rotate-90" />
              Add Wallet
            </TabsTrigger>
          </TabsList>

          <TabsContent value="wallets" className="space-y-6">
            <WalletConnectionManager
              wallets={wallets}
              onWalletUpdate={handleWalletAdded}
              onDisconnect={handleDisconnect}
              loading={loading}
            />
          </TabsContent>

          <TabsContent value="add" className="space-y-6">
            <AddWalletDialog onWalletAdded={handleWalletAdded} />
          </TabsContent>
        </Tabs>
      </div>
    </DashboardLayout>
  )
}

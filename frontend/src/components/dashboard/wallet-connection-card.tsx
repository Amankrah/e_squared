"use client"

import { useState } from "react"
import { Wallet, Trash2, Eye, EyeOff, Copy, CheckCircle2, AlertCircle, Loader2 } from "lucide-react"
import { type WalletConnection } from "@/lib/api"
import { apiClient } from "@/lib/api"

interface WalletConnectionCardProps {
  wallet: WalletConnection
  onUpdate: () => void
  onDisconnect: (id: string) => void
}

export function WalletConnectionCard({ wallet, onUpdate, onDisconnect }: WalletConnectionCardProps) {
  const [showBalance, setShowBalance] = useState(false)
  const [balance, setBalance] = useState<string | null>(null)
  const [loadingBalance, setLoadingBalance] = useState(false)
  const [copied, setCopied] = useState(false)
  const [password, setPassword] = useState("")
  const [showPasswordPrompt, setShowPasswordPrompt] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const getBlockchainIcon = (network: string) => {
    const normalized = network.toLowerCase()
    if (normalized === 'ethereum') return '⟠'
    if (normalized === 'bnbchain' || normalized === 'bsc') return '🔶'
    if (normalized === 'solana') return '◎'
    return '🔷'
  }

  const getBlockchainColor = (network: string) => {
    const normalized = network.toLowerCase()
    if (normalized === 'ethereum') return 'from-blue-500/20 to-blue-600/10 border-blue-400/30'
    if (normalized === 'bnbchain' || normalized === 'bsc') return 'from-yellow-500/20 to-yellow-600/10 border-yellow-400/30'
    if (normalized === 'solana') return 'from-purple-500/20 to-purple-600/10 border-purple-400/30'
    return 'from-gray-500/20 to-gray-600/10 border-gray-400/30'
  }

  const getStatusBadge = () => {
    if (wallet.connection_status === 'connected') {
      return (
        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-emerald-500/20 text-emerald-300 border border-emerald-400/30">
          <CheckCircle2 className="w-3 h-3 mr-1" />
          Connected
        </span>
      )
    }
    if (wallet.connection_status === 'error') {
      return (
        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-red-500/20 text-red-300 border border-red-400/30">
          <AlertCircle className="w-3 h-3 mr-1" />
          Error
        </span>
      )
    }
    return (
      <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-500/20 text-gray-300 border border-gray-400/30">
        <Loader2 className="w-3 h-3 mr-1 animate-spin" />
        Pending
      </span>
    )
  }

  const handleCopyAddress = async () => {
    try {
      await navigator.clipboard.writeText(wallet.wallet_address)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy address:', err)
    }
  }

  const handleToggleBalance = async () => {
    if (showBalance) {
      setShowBalance(false)
      setBalance(null)
      return
    }

    // Show password prompt
    setShowPasswordPrompt(true)
  }

  const handleFetchBalance = async () => {
    if (!password) {
      setError('Password is required')
      return
    }

    try {
      setError(null)
      setLoadingBalance(true)
      const response = await apiClient.getWalletBalance(wallet.id, password)
      setBalance(response.balance)
      setShowBalance(true)
      setShowPasswordPrompt(false)
      setPassword("")
    } catch (err: any) {
      setError(err.message || 'Failed to fetch balance')
    } finally {
      setLoadingBalance(false)
    }
  }

  const formatAddress = (address: string) => {
    if (address.length < 12) return address
    return `${address.slice(0, 6)}...${address.slice(-4)}`
  }

  return (
    <div className={`border rounded-xl bg-gradient-to-br ${getBlockchainColor(wallet.blockchain_network)} backdrop-blur-xl p-6 hover:shadow-lg transition-all duration-300 hover:scale-[1.02]`}>
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center space-x-3">
          <div className="p-3 bg-white/10 rounded-xl backdrop-blur-sm">
            <span className="text-2xl">{getBlockchainIcon(wallet.blockchain_network)}</span>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white">{wallet.display_name}</h3>
            <p className="text-sm text-gray-300 capitalize">{wallet.blockchain_network}</p>
          </div>
        </div>
        {getStatusBadge()}
      </div>

      {/* Wallet Address */}
      <div className="mb-4 p-3 bg-black/20 rounded-lg">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs text-gray-400 mb-1">Wallet Address</p>
            <p className="text-sm font-mono text-white">{formatAddress(wallet.wallet_address)}</p>
          </div>
          <button
            onClick={handleCopyAddress}
            className="p-2 hover:bg-white/10 rounded-lg transition-colors"
            title="Copy full address"
          >
            {copied ? (
              <CheckCircle2 className="w-4 h-4 text-emerald-400" />
            ) : (
              <Copy className="w-4 h-4 text-gray-300" />
            )}
          </button>
        </div>
      </div>

      {/* Balance Section */}
      <div className="mb-4">
        {showBalance && balance !== null ? (
          <div className="p-3 bg-emerald-500/10 rounded-lg border border-emerald-400/20">
            <p className="text-xs text-gray-400 mb-1">Balance</p>
            <p className="text-lg font-bold text-white">{balance}</p>
          </div>
        ) : showPasswordPrompt ? (
          <div className="space-y-3">
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter your password"
              className="w-full px-4 py-2 bg-black/20 border border-white/10 rounded-lg text-white placeholder-gray-400 focus:outline-none focus:border-purple-500/50"
              onKeyDown={(e) => e.key === 'Enter' && handleFetchBalance()}
            />
            {error && (
              <p className="text-xs text-red-400">{error}</p>
            )}
            <div className="flex space-x-2">
              <button
                onClick={handleFetchBalance}
                disabled={loadingBalance}
                className="flex-1 px-4 py-2 bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 rounded-lg text-white text-sm font-medium disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {loadingBalance ? (
                  <Loader2 className="w-4 h-4 mx-auto animate-spin" />
                ) : (
                  'Fetch Balance'
                )}
              </button>
              <button
                onClick={() => {
                  setShowPasswordPrompt(false)
                  setPassword("")
                  setError(null)
                }}
                className="px-4 py-2 bg-white/5 hover:bg-white/10 rounded-lg text-white text-sm font-medium"
              >
                Cancel
              </button>
            </div>
          </div>
        ) : null}
      </div>

      {/* Actions */}
      <div className="flex space-x-2 pt-4 border-t border-white/10">
        <button
          onClick={handleToggleBalance}
          className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 bg-white/5 hover:bg-white/10 rounded-lg text-white text-sm font-medium transition-colors"
        >
          {showBalance ? (
            <>
              <EyeOff className="w-4 h-4" />
              <span>Hide Balance</span>
            </>
          ) : (
            <>
              <Eye className="w-4 h-4" />
              <span>View Balance</span>
            </>
          )}
        </button>
        <button
          onClick={() => onDisconnect(wallet.id)}
          className="p-2 bg-red-500/10 hover:bg-red-500/20 rounded-lg text-red-400 hover:text-red-300 transition-colors"
          title="Disconnect wallet"
        >
          <Trash2 className="w-4 h-4" />
        </button>
      </div>

      {wallet.last_error && (
        <div className="mt-4 p-3 bg-red-500/10 rounded-lg border border-red-400/20">
          <p className="text-xs text-red-300">{wallet.last_error}</p>
        </div>
      )}
    </div>
  )
}

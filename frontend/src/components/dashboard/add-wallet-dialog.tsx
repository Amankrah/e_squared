"use client"

import { useState } from "react"
import { Wallet, Plus, AlertCircle, CheckCircle2, Loader2, Shield, Eye, EyeOff } from "lucide-react"
import { apiClient } from "@/lib/api"

interface AddWalletDialogProps {
  onWalletAdded: () => void
}

export function AddWalletDialog({ onWalletAdded }: AddWalletDialogProps) {
  const [blockchain, setBlockchain] = useState<string>("")
  const [displayName, setDisplayName] = useState("")
  const [privateKey, setPrivateKey] = useState("")
  const [password, setPassword] = useState("")
  const [showPrivateKey, setShowPrivateKey] = useState(false)
  const [showPassword, setShowPassword] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const blockchains = [
    { value: "ethereum", label: "Ethereum", icon: "⟠", description: "Trade on Uniswap and other ETH DEXs" },
    { value: "bnbchain", label: "BNB Chain", icon: "🔶", description: "Trade on PancakeSwap and BSC DEXs" },
    { value: "solana", label: "Solana", icon: "◎", description: "Trade on Raydium, Jupiter and Solana DEXs" }
  ]

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)
    setSuccess(false)

    // Validation
    if (!blockchain) {
      setError("Please select a blockchain")
      return
    }
    if (!displayName.trim()) {
      setError("Please enter a wallet name")
      return
    }
    if (!privateKey.trim()) {
      setError("Please enter your private key")
      return
    }
    if (!password.trim()) {
      setError("Please enter your password for encryption")
      return
    }

    try {
      setLoading(true)
      await apiClient.createWalletConnection({
        blockchain_network: blockchain,
        display_name: displayName.trim(),
        private_key: privateKey.trim(),
        password: password
      })

      setSuccess(true)
      // Reset form
      setBlockchain("")
      setDisplayName("")
      setPrivateKey("")
      setPassword("")

      // Notify parent
      setTimeout(() => {
        onWalletAdded()
        setSuccess(false)
      }, 1500)
    } catch (err: any) {
      setError(err.message || 'Failed to add wallet connection')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8">
      <div className="mb-6">
        <div className="flex items-center space-x-3 mb-2">
          <div className="p-2 bg-purple-500/20 rounded-lg">
            <Plus className="w-6 h-6 text-purple-300" />
          </div>
          <h2 className="text-2xl font-bold text-white">Add Crypto Wallet</h2>
        </div>
        <p className="text-gray-300 ml-11">
          Connect your crypto wallet to trade on decentralized exchanges (DEX)
        </p>
      </div>

      {/* Security Notice */}
      <div className="mb-6 p-4 bg-blue-500/10 rounded-xl border border-blue-400/20">
        <div className="flex items-start space-x-3">
          <Shield className="w-5 h-5 text-blue-300 mt-0.5" />
          <div>
            <h4 className="text-sm font-semibold text-blue-200 mb-1">Your Keys, Your Crypto</h4>
            <p className="text-xs text-blue-300/80">
              Your private key is encrypted with your password using AES-256 before storage.
              We never have access to your unencrypted keys.
            </p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-6">
        {/* Blockchain Selection */}
        <div>
          <label className="block text-sm font-medium text-gray-200 mb-3">
            Select Blockchain
          </label>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
            {blockchains.map((chain) => (
              <button
                key={chain.value}
                type="button"
                onClick={() => setBlockchain(chain.value)}
                className={`p-4 rounded-xl border-2 transition-all ${
                  blockchain === chain.value
                    ? 'border-purple-500/50 bg-purple-500/20'
                    : 'border-white/10 bg-white/5 hover:border-purple-500/30 hover:bg-purple-500/10'
                }`}
              >
                <div className="flex items-center space-x-3 mb-2">
                  <span className="text-2xl">{chain.icon}</span>
                  <span className="font-semibold text-white">{chain.label}</span>
                </div>
                <p className="text-xs text-gray-400 text-left">{chain.description}</p>
              </button>
            ))}
          </div>
        </div>

        {/* Wallet Name */}
        <div>
          <label htmlFor="displayName" className="block text-sm font-medium text-gray-200 mb-2">
            Wallet Name
          </label>
          <input
            id="displayName"
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="e.g., My Trading Wallet"
            className="w-full px-4 py-3 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:border-purple-500/50 transition-colors"
            required
          />
        </div>

        {/* Private Key */}
        <div>
          <label htmlFor="privateKey" className="block text-sm font-medium text-gray-200 mb-2">
            Private Key
          </label>
          <div className="relative">
            <input
              id="privateKey"
              type={showPrivateKey ? "text" : "password"}
              value={privateKey}
              onChange={(e) => setPrivateKey(e.target.value)}
              placeholder="Your wallet private key (will be encrypted)"
              className="w-full px-4 py-3 pr-12 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:border-purple-500/50 transition-colors font-mono text-sm"
              required
            />
            <button
              type="button"
              onClick={() => setShowPrivateKey(!showPrivateKey)}
              className="absolute right-3 top-1/2 -translate-y-1/2 p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              {showPrivateKey ? (
                <EyeOff className="w-4 h-4 text-gray-400" />
              ) : (
                <Eye className="w-4 h-4 text-gray-400" />
              )}
            </button>
          </div>
          <p className="text-xs text-gray-400 mt-2">
            For Ethereum/BNB: 0x... format. For Solana: Base58 or hex format
          </p>
        </div>

        {/* Password */}
        <div>
          <label htmlFor="password" className="block text-sm font-medium text-gray-200 mb-2">
            Your Password
          </label>
          <div className="relative">
            <input
              id="password"
              type={showPassword ? "text" : "password"}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Your account password for encryption"
              className="w-full px-4 py-3 pr-12 bg-black/20 border border-white/10 rounded-xl text-white placeholder-gray-400 focus:outline-none focus:border-purple-500/50 transition-colors"
              required
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 -translate-y-1/2 p-2 hover:bg-white/10 rounded-lg transition-colors"
            >
              {showPassword ? (
                <EyeOff className="w-4 h-4 text-gray-400" />
              ) : (
                <Eye className="w-4 h-4 text-gray-400" />
              )}
            </button>
          </div>
          <p className="text-xs text-gray-400 mt-2">
            Used to encrypt your private key before storage
          </p>
        </div>

        {/* Error Message */}
        {error && (
          <div className="p-4 bg-red-500/10 rounded-xl border border-red-400/20 flex items-start space-x-3">
            <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
            <p className="text-sm text-red-300">{error}</p>
          </div>
        )}

        {/* Success Message */}
        {success && (
          <div className="p-4 bg-emerald-500/10 rounded-xl border border-emerald-400/20 flex items-start space-x-3">
            <CheckCircle2 className="w-5 h-5 text-emerald-400 flex-shrink-0 mt-0.5" />
            <p className="text-sm text-emerald-300">Wallet connected successfully!</p>
          </div>
        )}

        {/* Submit Button */}
        <button
          type="submit"
          disabled={loading}
          className="w-full py-4 bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 rounded-xl text-white font-semibold disabled:opacity-50 disabled:cursor-not-allowed transition-all hover:shadow-lg hover:shadow-purple-500/25"
        >
          {loading ? (
            <span className="flex items-center justify-center space-x-2">
              <Loader2 className="w-5 h-5 animate-spin" />
              <span>Connecting Wallet...</span>
            </span>
          ) : (
            <span className="flex items-center justify-center space-x-2">
              <Wallet className="w-5 h-5" />
              <span>Connect Wallet</span>
            </span>
          )}
        </button>
      </form>

      {/* Warning */}
      <div className="mt-6 p-4 bg-yellow-500/10 rounded-xl border border-yellow-400/20">
        <div className="flex items-start space-x-3">
          <AlertCircle className="w-5 h-5 text-yellow-300 flex-shrink-0 mt-0.5" />
          <div>
            <h4 className="text-sm font-semibold text-yellow-200 mb-1">Important</h4>
            <p className="text-xs text-yellow-300/80">
              Never share your private key with anyone. Make sure you're on the correct website before entering sensitive information.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

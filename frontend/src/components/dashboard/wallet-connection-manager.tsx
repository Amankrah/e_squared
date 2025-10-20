"use client"

import { Wallet, AlertCircle, Loader2 } from "lucide-react"
import { WalletConnectionCard } from "./wallet-connection-card"
import { type WalletConnection } from "@/lib/api"

interface WalletConnectionManagerProps {
  wallets: WalletConnection[]
  onWalletUpdate: () => void
  onDisconnect: (id: string) => void
  loading: boolean
}

export function WalletConnectionManager({
  wallets,
  onWalletUpdate,
  onDisconnect,
  loading
}: WalletConnectionManagerProps) {
  if (loading) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12">
        <div className="flex flex-col items-center justify-center space-y-4">
          <div className="p-4 bg-purple-500/20 rounded-2xl">
            <Loader2 className="w-8 h-8 text-purple-300 animate-spin" />
          </div>
          <p className="text-gray-300">Loading your wallet connections...</p>
        </div>
      </div>
    )
  }

  if (wallets.length === 0) {
    return (
      <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12">
        <div className="flex flex-col items-center justify-center space-y-4 max-w-md mx-auto text-center">
          <div className="p-4 bg-purple-500/20 rounded-2xl">
            <Wallet className="w-12 h-12 text-purple-300" />
          </div>
          <h3 className="text-xl font-semibold text-white">No Wallets Connected</h3>
          <p className="text-gray-300">
            Connect your crypto wallet to start trading on decentralized exchanges.
            Your private keys are encrypted and stored securely.
          </p>
          <div className="mt-4 p-4 bg-blue-500/10 rounded-xl border border-blue-400/20 text-left w-full">
            <h4 className="text-sm font-semibold text-blue-200 mb-2">Supported Blockchains</h4>
            <ul className="space-y-2 text-sm text-blue-300/80">
              <li className="flex items-center space-x-2">
                <span>⟠</span>
                <span>Ethereum - Trade on Uniswap V3</span>
              </li>
              <li className="flex items-center space-x-2">
                <span>🔶</span>
                <span>BNB Chain - Trade on PancakeSwap V3</span>
              </li>
              <li className="flex items-center space-x-2">
                <span>◎</span>
                <span>Solana - Trade on Raydium & Jupiter</span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    )
  }

  // Group wallets by blockchain
  const groupedWallets = wallets.reduce((acc, wallet) => {
    const network = wallet.blockchain_network.toLowerCase()
    if (!acc[network]) {
      acc[network] = []
    }
    acc[network].push(wallet)
    return acc
  }, {} as Record<string, WalletConnection[]>)

  const blockchainLabels: Record<string, { label: string; icon: string }> = {
    ethereum: { label: "Ethereum", icon: "⟠" },
    bnbchain: { label: "BNB Chain", icon: "🔶" },
    bsc: { label: "BNB Chain", icon: "🔶" },
    solana: { label: "Solana", icon: "◎" }
  }

  return (
    <div className="space-y-8">
      {Object.entries(groupedWallets).map(([network, networkWallets]) => {
        const blockchainInfo = blockchainLabels[network] || { label: network, icon: "🔷" }

        return (
          <div key={network}>
            <div className="flex items-center space-x-2 mb-4">
              <span className="text-2xl">{blockchainInfo.icon}</span>
              <h3 className="text-lg font-semibold text-white capitalize">
                {blockchainInfo.label}
              </h3>
              <span className="text-sm text-gray-400">
                ({networkWallets.length} wallet{networkWallets.length !== 1 ? 's' : ''})
              </span>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {networkWallets.map((wallet) => (
                <WalletConnectionCard
                  key={wallet.id}
                  wallet={wallet}
                  onUpdate={onWalletUpdate}
                  onDisconnect={onDisconnect}
                />
              ))}
            </div>
          </div>
        )
      })}

      {/* Info Section */}
      <div className="border border-[rgba(147,51,234,0.2)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.05)] to-[rgba(147,51,234,0.01)] backdrop-blur-xl p-6">
        <div className="flex items-start space-x-3">
          <AlertCircle className="w-5 h-5 text-purple-300 flex-shrink-0 mt-0.5" />
          <div>
            <h4 className="text-sm font-semibold text-purple-200 mb-2">Security Notice</h4>
            <p className="text-sm text-gray-300">
              Your private keys are encrypted with AES-256 using your password before being stored.
              We never have access to your unencrypted keys. Always verify the wallet address after connection.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}

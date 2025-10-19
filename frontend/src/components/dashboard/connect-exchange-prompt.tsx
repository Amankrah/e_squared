"use client"

import { Link2, TrendingUp, Shield, Zap, ArrowRight, Wallet } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { useState } from "react"
import { ExchangeConnectionDialog } from "./exchange-connection-dialog"

export function ConnectExchangePrompt() {
  const [showConnectionDialog, setShowConnectionDialog] = useState(false)

  const features = [
    {
      icon: TrendingUp,
      title: "Automated Trading",
      description: "Execute DCA, Grid, and SMA strategies across CEX and DEX platforms"
    },
    {
      icon: Shield,
      title: "Secure & Non-Custodial",
      description: "API keys encrypted on our servers, DEX via your own wallet - we never hold your funds"
    },
    {
      icon: Zap,
      title: "Real-time Execution",
      description: "Monitor markets and execute trades instantly across all connected platforms"
    }
  ]

  return (
    <>
      <Card className="relative overflow-hidden border-purple-500/20 bg-gradient-to-br from-purple-500/5 via-transparent to-pink-500/5">
        <CardHeader className="text-center pb-2">
          <div className="w-16 h-16 mx-auto mb-4 bg-purple-500/20 rounded-2xl flex items-center justify-center">
            <Link2 className="w-8 h-8 text-purple-400" />
          </div>
          <CardTitle className="text-2xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
            Connect Your Trading Platform
          </CardTitle>
          <CardDescription className="text-gray-300 mt-2">
            Connect centralized exchanges or decentralized wallets to start trading
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-6 pt-6">
          <Alert className="border-purple-500/30 bg-purple-500/10">
            <AlertDescription className="text-purple-200">
              <strong>Welcome to E²!</strong> Connect CEX via API keys or DEX via wallet to enable automated strategies (DCA, Grid Trading, SMA Crossover), portfolio tracking, and real-time execution.
            </AlertDescription>
          </Alert>

          <div className="space-y-4">
            {features.map((feature, index) => (
              <div key={index} className="flex gap-3">
                <div className="flex-shrink-0 w-10 h-10 bg-purple-500/20 rounded-lg flex items-center justify-center">
                  <feature.icon className="w-5 h-5 text-purple-400" />
                </div>
                <div className="flex-1">
                  <h3 className="font-semibold text-white mb-1">{feature.title}</h3>
                  <p className="text-sm text-gray-400">{feature.description}</p>
                </div>
              </div>
            ))}
          </div>

          <div className="pt-4 space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <Button
                onClick={() => setShowConnectionDialog(true)}
                className="bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white"
                size="lg"
              >
                <Link2 className="w-4 h-4 mr-2" />
                CEX API
              </Button>

              <Button
                onClick={() => {/* TODO: Implement wallet connection */}}
                className="bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-600 hover:to-blue-600 text-white"
                size="lg"
              >
                <Wallet className="w-4 h-4 mr-2" />
                DEX Wallet
              </Button>
            </div>

            <div className="flex flex-wrap gap-2 justify-center pt-2">
              <Badge variant="outline" className="text-xs border-purple-400/30 text-purple-300">
                Binance
              </Badge>
              <Badge variant="outline" className="text-xs border-purple-400/30 text-purple-300">
                Coinbase
              </Badge>
              <Badge variant="outline" className="text-xs border-cyan-400/30 text-cyan-300">
                Uniswap
              </Badge>
              <Badge variant="outline" className="text-xs border-cyan-400/30 text-cyan-300">
                PancakeSwap
              </Badge>
              <Badge variant="outline" className="text-xs border-purple-400/30 text-purple-300">
                +More
              </Badge>
            </div>

            <p className="text-xs text-gray-400 text-center">
              API keys encrypted · Non-custodial wallet control · Your funds stay secure
            </p>
          </div>

          <div className="absolute top-0 right-0 w-32 h-32 bg-purple-500/10 rounded-full blur-3xl" />
          <div className="absolute bottom-0 left-0 w-40 h-40 bg-pink-500/10 rounded-full blur-3xl" />
        </CardContent>
      </Card>

      {showConnectionDialog && (
        <ExchangeConnectionDialog
          isOpen={showConnectionDialog}
          onClose={() => setShowConnectionDialog(false)}
          onSuccess={() => {
            setShowConnectionDialog(false)
            // Reload the page or trigger a refresh of exchange connections
            window.location.reload()
          }}
        />
      )}
    </>
  )
}
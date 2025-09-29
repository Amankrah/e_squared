"use client"

import { useEffect } from "react"
import { useRouter } from "next/navigation"
import { DashboardLayout } from "@/components/dashboard/dashboard-layout"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ArrowRight, Zap } from "lucide-react"
import Link from "next/link"

export default function StrategiesPage() {
  const router = useRouter()

  // Redirect to unified strategies page after a short delay
  useEffect(() => {
    const timer = setTimeout(() => {
      router.push('/dashboard/strategies/unified')
    }, 3000)

    return () => clearTimeout(timer)
  }, [router])

  return (
    <DashboardLayout>
      <div className="flex items-center justify-center min-h-[60vh]">
        <Card className="bg-gradient-to-br from-purple-500/20 to-blue-500/20 backdrop-blur-xl border border-white/10 max-w-2xl w-full">
          <div className="absolute inset-0 bg-white/5 backdrop-blur-sm rounded-lg" />
          
          <div className="relative z-10">
            <CardHeader className="text-center space-y-6">
              <div className="mx-auto w-16 h-16 bg-gradient-to-br from-purple-600 to-blue-600 rounded-full flex items-center justify-center">
                <Zap className="h-8 w-8 text-white animate-pulse" />
              </div>
              
              <div className="space-y-2">
                <CardTitle className="text-3xl font-bold text-white/90">
                  Strategy Management Upgraded! 
                </CardTitle>
                <CardDescription className="text-white/60 text-lg">
                  We&apos;ve enhanced your trading experience with a unified multi-strategy platform
                </CardDescription>
              </div>
            </CardHeader>

            <CardContent className="space-y-6">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-white/5 rounded-lg p-4 space-y-2">
                  <h3 className="font-semibold text-white/90">🎯 Multiple Strategy Types</h3>
                  <p className="text-sm text-white/70">
                    DCA, Grid Trading, SMA Crossover, RSI, and MACD strategies all in one place
                  </p>
                </div>
                
                <div className="bg-white/5 rounded-lg p-4 space-y-2">
                  <h3 className="font-semibold text-white/90">📊 Advanced Backtesting</h3>
                  <p className="text-sm text-white/70">
                    Test any strategy with historical data before going live
                  </p>
                </div>
                
                <div className="bg-white/5 rounded-lg p-4 space-y-2">
                  <h3 className="font-semibold text-white/90">🎨 Beautiful Interface</h3>
                  <p className="text-sm text-white/70">
                    Enhanced glassmorphism design with better user experience
                  </p>
                </div>
                
                <div className="bg-white/5 rounded-lg p-4 space-y-2">
                  <h3 className="font-semibold text-white/90">⚡ Performance Focused</h3>
                  <p className="text-sm text-white/70">
                    Faster loading, better filtering, and real-time updates
                  </p>
                </div>
              </div>

              <div className="text-center space-y-4">
                <p className="text-white/60">
                  Redirecting to the new unified strategy dashboard...
                </p>
                
                <div className="flex justify-center space-x-3">
                  <Link href="/dashboard/strategies/unified">
                    <Button className="bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-500 hover:to-blue-500 text-white">
                      <ArrowRight className="mr-2 h-4 w-4" />
                      Go to New Interface
                    </Button>
                  </Link>
                </div>
                
                <div className="w-full bg-white/10 rounded-full h-2 mt-4">
                  <div className="bg-gradient-to-r from-purple-600 to-blue-600 h-2 rounded-full animate-pulse" style={{ width: '100%' }}></div>
                </div>
              </div>
            </CardContent>
          </div>
        </Card>
      </div>
    </DashboardLayout>
  )
}
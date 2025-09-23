import Link from "next/link"
import { ArrowRight, Shield, TrendingUp, Users, Star, BarChart3 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function Home() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      {/* Enhanced Hero Section with Glass Morphism */}
      <section className="relative min-h-screen flex items-center justify-center bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden">
        {/* Animated Background Elements */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse"></div>
          <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/20 to-purple-400/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-indigo-400/20 to-blue-400/20 rounded-full blur-3xl animate-pulse delay-500"></div>
        </div>

        {/* Grid Pattern Overlay */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

        <div className="relative container px-4 py-20">
          <div className="grid gap-16 lg:grid-cols-2 lg:gap-20 items-center">
            {/* Left Content with Glass Morphism Cards */}
            <div className="space-y-8">
              {/* Main Glass Card */}
              <div className="relative">
                <div className="h-auto w-full border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8 text-white">
                  <div className="space-y-6">

                    <h1 className="text-4xl lg:text-6xl xl:text-7xl font-bold tracking-tight leading-tight">
                      <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                        Trade, Don't{" "}
                      </span>
                      <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-600 bg-clip-text text-transparent">
                        Gamble
                      </span>
                    </h1>

                    <p className="text-lg text-gray-200 leading-relaxed max-w-2xl">
                      Master crypto trading with confidence—build winning strategies using our beginner-friendly
                      platform. No experience? Our AI automates risk management and guides you to profits, step-by-step, on your favorite exchanges.
                    </p>
                  </div>

                  {/* Enhanced CTA Button - Primary Focus */}
                  <div className="flex justify-center mt-8">
                    <Link href="/signup">
                      <button className="relative h-fit w-fit px-12 py-4 border-2 border-purple-400/60 rounded-full flex justify-center items-center gap-3 overflow-hidden group hover:scale-105 duration-300 backdrop-blur-xl bg-gradient-to-r from-purple-600 to-pink-600 hover:from-purple-500 hover:to-pink-500 transition-all shadow-2xl shadow-purple-500/25 hover:shadow-purple-500/40">
                        {/* Pulsing glow effect */}
                        <div className="absolute inset-0 rounded-full bg-gradient-to-r from-purple-600 to-pink-600 opacity-75 animate-pulse"></div>
                        <div className="absolute inset-0 rounded-full bg-gradient-to-r from-purple-600 to-pink-600 blur-lg opacity-50"></div>

                        {/* Button content */}
                        <div className="relative flex items-center gap-3">
                          <span className="text-white font-bold text-lg">Get Started Free</span>
                          <ArrowRight className="w-6 h-6 group-hover:translate-x-1 duration-300 text-white" />
                        </div>
                      </button>
                    </Link>
                  </div>

                </div>
              </div>

              {/* Trust Indicators */}
              <div className="flex flex-wrap gap-4">
                <div className="border border-[rgba(16,185,129,0.3)] rounded-2xl bg-gradient-to-br from-[rgba(16,185,129,0.15)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-4 flex items-center space-x-3">
                  <div className="p-2 bg-emerald-500/20 rounded-xl">
                    <Shield className="h-5 w-5 text-emerald-300" />
                  </div>
                  <span className="font-medium text-emerald-100">Bank-level Security</span>
                </div>

                <div className="border border-[rgba(99,102,241,0.3)] rounded-2xl bg-gradient-to-br from-[rgba(99,102,241,0.15)] to-[rgba(99,102,241,0.02)] backdrop-blur-xl p-4 flex items-center space-x-3">
                  <div className="p-2 bg-indigo-500/20 rounded-xl">
                    <Users className="h-5 w-5 text-indigo-300" />
                  </div>
                  <span className="font-medium text-indigo-100">Join Thousands</span>
                </div>
              </div>
            </div>
            
            {/* Expanded Background Chart */}
            <div className="absolute inset-0 flex items-end justify-end opacity-15 overflow-hidden" style={{ transform: 'translate(25%, 15%)' }}>
              {/* Large Chart Background Covering More Area */}
              <svg viewBox="0 0 800 600" className="w-full h-full min-w-[120%] min-h-[120%]">
                {/* Minimal Grid */}
                <defs>
                  <pattern id="subtleGrid" width="40" height="30" patternUnits="userSpaceOnUse">
                    <path d="M 40 0 L 0 0 0 30" fill="none" stroke="rgba(255,255,255,0.05)" strokeWidth="0.5"/>
                  </pattern>
                </defs>
                <rect width="800" height="600" fill="url(#subtleGrid)"/>

                {/* Resistance Line (Horizontal) */}
                <line x1="50" y1="120" x2="650" y2="120" stroke="rgba(220, 38, 38, 0.3)" strokeWidth="2" strokeDasharray="8,4"/>

                {/* Support Line (Ascending) */}
                <line x1="50" y1="420" x2="650" y2="180" stroke="rgba(34, 197, 94, 0.3)" strokeWidth="2" strokeDasharray="8,4"/>

                {/* Many Candlesticks Forming Ascending Triangle */}
                {/* Early Formation - Lower lows moving higher */}
                <rect x="70" y="400" width="8" height="20" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="74" y1="390" x2="74" y2="425" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="90" y="380" width="8" height="15" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="94" y1="370" x2="94" y2="395" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="110" y="350" width="8" height="25" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="114" y1="340" x2="114" y2="380" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="130" y="320" width="8" height="30" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="134" y1="310" x2="134" y2="355" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                {/* First resistance test */}
                <rect x="150" y="130" width="8" height="180" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="154" y1="120" x2="154" y2="320" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="170" y="140" width="8" height="30" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="174" y1="125" x2="174" y2="175" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                {/* Higher lows continuing */}
                <rect x="190" y="300" width="8" height="25" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="194" y1="290" x2="194" y2="330" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="210" y="280" width="8" height="20" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="214" y1="270" x2="214" y2="305" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="230" y="260" width="8" height="35" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="234" y1="250" x2="234" y2="300" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                {/* More resistance tests */}
                <rect x="250" y="135" width="8" height="40" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="254" y1="120" x2="254" y2="180" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="270" y="145" width="8" height="25" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="274" y1="125" x2="274" y2="175" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                {/* Continuing pattern */}
                <rect x="290" y="240" width="8" height="30" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="294" y1="230" x2="294" y2="275" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="310" y="220" width="8" height="20" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="314" y1="210" x2="314" y2="245" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="330" y="140" width="8" height="35" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="334" y1="120" x2="334" y2="180" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="350" y="150" width="8" height="20" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="354" y1="130" x2="354" y2="175" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                {/* Tightening range */}
                <rect x="370" y="200" width="8" height="25" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="374" y1="190" x2="374" y2="230" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="390" y="190" width="8" height="15" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="394" y1="180" x2="394" y2="210" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="410" y="145" width="8" height="30" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="414" y1="125" x2="414" y2="180" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="430" y="155" width="8" height="15" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="434" y1="135" x2="434" y2="175" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                {/* Final compression */}
                <rect x="450" y="170" width="8" height="20" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="454" y1="160" x2="454" y2="195" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="470" y="160" width="8" height="10" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="474" y1="150" x2="474" y2="175" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="490" y="150" width="8" height="15" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="494" y1="140" x2="494" y2="170" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                <rect x="510" y="145" width="8" height="8" fill="rgba(220, 38, 38, 0.4)"/>
                <line x1="514" y1="135" x2="514" y2="158" stroke="rgba(220, 38, 38, 0.4)" strokeWidth="1"/>

                <rect x="530" y="140" width="8" height="10" fill="rgba(34, 197, 94, 0.4)"/>
                <line x1="534" y1="130" x2="534" y2="155" stroke="rgba(34, 197, 94, 0.4)" strokeWidth="1"/>

                {/* BREAKOUT CANDLES - More prominent */}
                <rect x="550" y="80" width="8" height="60" fill="rgba(34, 197, 94, 0.6)"/>
                <line x1="554" y1="70" x2="554" y2="150" stroke="rgba(34, 197, 94, 0.6)" strokeWidth="1"/>

                <rect x="570" y="60" width="8" height="70" fill="rgba(34, 197, 94, 0.6)"/>
                <line x1="574" y1="50" x2="574" y2="140" stroke="rgba(34, 197, 94, 0.6)" strokeWidth="1"/>

                <rect x="590" y="40" width="8" height="80" fill="rgba(34, 197, 94, 0.6)"/>
                <line x1="594" y1="30" x2="594" y2="130" stroke="rgba(34, 197, 94, 0.6)" strokeWidth="1"/>

                {/* Breakout Arrow */}
                <path d="M 610 70 L 620 60 L 610 50 L 610 55 L 595 55 L 595 65 L 610 65 Z"
                      fill="rgba(34, 197, 94, 0.5)"/>
              </svg>
            </div>
          </div>
        </div>
      </section>

      {/* Enhanced Features Section with Glass Morphism */}
      <section className="relative py-20 md:py-32 bg-gradient-to-b from-[#16213e] to-[#0F0C29] dark:from-[#16213e] dark:to-[#0a0a0a] overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/3 left-1/4 w-64 h-64 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/3 right-1/4 w-80 h-80 bg-gradient-to-r from-blue-500/10 to-purple-500/10 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          {/* Section Header */}
          <div className="text-center space-y-8 mb-20">

            <h2 className="text-4xl lg:text-5xl xl:text-6xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Everything You Need to
              </span>
              <br />
              <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-600 bg-clip-text text-transparent">
                Succeed
              </span>
            </h2>

            <p className="text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed">
              Our platform combines powerful tools with beginner-friendly design
              to help you build and execute profitable trading strategies.
            </p>
          </div>

          {/* Feature Cards Grid */}
          <div className="grid gap-8 md:grid-cols-3">
            {/* Strategy Builder Card */}
            <div className="group">
              <div className="h-auto min-h-[320px] border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-10 text-white transition-all duration-500 hover:border-[rgba(147,51,234,0.4)] hover:from-[rgba(147,51,234,0.15)] hover:to-[rgba(147,51,234,0.05)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-2xl w-fit backdrop-blur-sm border border-purple-400/20 group-hover:scale-110 transition-transform duration-300">
                    <TrendingUp className="h-8 w-8 text-purple-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">Strategy Builder</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Create custom trading strategies with our intuitive drag-and-drop interface.
                    No coding required - just point, click, and profit.
                  </p>
                </div>
              </div>
            </div>

            {/* Risk Management Card */}
            <div className="group">
              <div className="h-auto min-h-[320px] border-2 border-[rgba(244,63,94,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(244,63,94,0.1)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl p-10 text-white transition-all duration-500 hover:border-[rgba(244,63,94,0.4)] hover:from-[rgba(244,63,94,0.15)] hover:to-[rgba(244,63,94,0.05)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-pink-500/30 to-rose-500/30 rounded-2xl w-fit backdrop-blur-sm border border-pink-400/20 group-hover:scale-110 transition-transform duration-300">
                    <Shield className="h-8 w-8 text-pink-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">Risk Management</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Built-in stop-loss, take-profit, and position sizing tools to protect
                    your capital and maximize returns with confidence.
                  </p>
                </div>
              </div>
            </div>

            {/* Exchange Integration Card */}
            <div className="group">
              <div className="h-auto min-h-[320px] border-2 border-[rgba(16,185,129,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-10 text-white transition-all duration-500 hover:border-[rgba(16,185,129,0.4)] hover:from-[rgba(16,185,129,0.15)] hover:to-[rgba(16,185,129,0.05)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-2xl w-fit backdrop-blur-sm border border-emerald-400/20 group-hover:scale-110 transition-transform duration-300">
                    <BarChart3 className="h-8 w-8 text-emerald-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">Exchange Integration</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Connect with major exchanges like Binance and Coinbase.
                    Execute trades directly from our platform with institutional-grade security.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Enhanced Testimonials Section with Glass Morphism */}
      <section className="relative py-20 md:py-32 bg-gradient-to-b from-[#0F0C29] to-[#24243e] dark:from-[#0a0a0a] dark:to-[#1a1a2e] overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 right-1/3 w-72 h-72 bg-gradient-to-r from-emerald-500/10 to-teal-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          {/* Section Header */}
          <div className="text-center space-y-8 mb-20">

            <h2 className="text-4xl lg:text-5xl xl:text-6xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                What Our Users Say
              </span>
            </h2>

            <p className="text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed">
              Join thousands of traders who've transformed their approach to cryptocurrency trading.
            </p>
          </div>

          {/* Testimonials Grid */}
          <div className="grid gap-8 md:grid-cols-3">
            {/* Testimonial 1 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(16,185,129,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(16,185,129,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  {/* Star Rating */}
                  <div className="flex items-center space-x-1">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="h-5 w-5 fill-amber-400 text-amber-400" />
                    ))}
                  </div>

                  {/* Quote */}
                  <p className="text-gray-200 leading-relaxed text-lg">
                    "E-Squared made trading simple for me as a complete beginner.
                    I went from losing money to consistent profits in just 3 months!"
                  </p>

                  {/* User Info */}
                  <div className="flex items-center space-x-4 pt-4">
                    <div className="h-12 w-12 rounded-full bg-gradient-to-br from-emerald-500/80 to-teal-600/80 flex items-center justify-center shadow-lg border border-emerald-400/30">
                      <span className="text-sm font-bold text-white">A</span>
                    </div>
                    <div>
                      <p className="font-semibold text-white">Alex Chen</p>
                      <p className="text-sm text-gray-400">Beginner Trader</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Testimonial 2 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(147,51,234,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  {/* Star Rating */}
                  <div className="flex items-center space-x-1">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="h-5 w-5 fill-amber-400 text-amber-400" />
                    ))}
                  </div>

                  {/* Quote */}
                  <p className="text-gray-200 leading-relaxed text-lg">
                    "The risk management tools saved me from huge losses during the market crash.
                    This platform is a game-changer!"
                  </p>

                  {/* User Info */}
                  <div className="flex items-center space-x-4 pt-4">
                    <div className="h-12 w-12 rounded-full bg-gradient-to-br from-purple-500/80 to-pink-600/80 flex items-center justify-center shadow-lg border border-purple-400/30">
                      <span className="text-sm font-bold text-white">S</span>
                    </div>
                    <div>
                      <p className="font-semibold text-white">Sarah Johnson</p>
                      <p className="text-sm text-gray-400">Day Trader</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Testimonial 3 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(59,130,246,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(59,130,246,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  {/* Star Rating */}
                  <div className="flex items-center space-x-1">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="h-5 w-5 fill-amber-400 text-amber-400" />
                    ))}
                  </div>

                  {/* Quote */}
                  <p className="text-gray-200 leading-relaxed text-lg">
                    "Finally, a platform that explains everything in plain English.
                    The AI guidance helped me understand market patterns."
                  </p>

                  {/* User Info */}
                  <div className="flex items-center space-x-4 pt-4">
                    <div className="h-12 w-12 rounded-full bg-gradient-to-br from-blue-500/80 to-cyan-600/80 flex items-center justify-center shadow-lg border border-blue-400/30">
                      <span className="text-sm font-bold text-white">M</span>
                    </div>
                    <div>
                      <p className="font-semibold text-white">Mike Rodriguez</p>
                      <p className="text-sm text-gray-400">Crypto Enthusiast</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Enhanced CTA Section with Glass Morphism */}
      <section className="relative py-20 md:py-32 bg-gradient-to-br from-[#24243e] via-[#302b63] to-[#0F0C29] dark:from-[#1a1a2e] dark:via-[#16213e] dark:to-[#0a0a0a] text-white overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-gradient-to-r from-purple-500/15 to-pink-500/15 rounded-full blur-3xl animate-pulse"></div>
          <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-gradient-to-r from-blue-500/15 to-purple-500/15 rounded-full blur-3xl animate-pulse delay-1000"></div>
          <div className="absolute top-3/4 left-1/2 w-64 h-64 bg-gradient-to-r from-emerald-500/10 to-teal-500/10 rounded-full blur-3xl animate-pulse delay-500"></div>
        </div>

        {/* Grid Pattern */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

        <div className="relative container px-4 text-center">
          <div className="max-w-4xl mx-auto">
            {/* Main CTA Card */}
            <div className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 lg:p-16">
              <div className="space-y-10">
                {/* Badge */}

                {/* Heading */}
                <h2 className="text-4xl lg:text-6xl xl:text-7xl font-bold tracking-tight leading-tight">
                  <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                    Ready to Start Trading
                  </span>
                  <br />
                  <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-600 bg-clip-text text-transparent">
                    Smarter?
                  </span>
                </h2>

                {/* Description */}
                <p className="text-xl text-gray-200 max-w-3xl mx-auto leading-relaxed">
                  Join thousands of successful traders. Start your free trial today -
                  no credit card required. Transform your trading with AI-powered strategies.
                </p>

                {/* CTA Buttons */}
                <div className="flex flex-col sm:flex-row gap-6 justify-center items-center">
                  <Link href="/signup">
                    <button className="h-fit w-fit px-10 py-4 border border-purple-400/50 rounded-full flex justify-center items-center gap-3 overflow-hidden group hover:translate-y-1 duration-200 backdrop-blur-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 transition-all text-lg font-semibold">
                      <span className="text-white">Start Free Trial</span>
                      <ArrowRight className="w-5 h-5 group-hover:translate-x-1 duration-300 text-white" />
                    </button>
                  </Link>

                  <Link href="/contact">
                    <button className="h-fit w-fit px-10 py-4 border border-white/30 rounded-full flex justify-center items-center gap-3 overflow-hidden group hover:translate-y-1 duration-200 backdrop-blur-xl bg-white/10 hover:bg-white/20 transition-all text-lg font-semibold">
                      <span className="text-white">Contact Sales</span>
                    </button>
                  </Link>
                </div>

                {/* Trust Indicators */}
                <div className="flex flex-wrap justify-center gap-8 text-sm">
                  <div className="flex items-center space-x-3">
                    <div className="w-3 h-3 bg-gradient-to-r from-emerald-400 to-emerald-500 rounded-full shadow-lg"></div>
                    <span className="text-gray-300 font-medium">No credit card required</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <div className="w-3 h-3 bg-gradient-to-r from-emerald-400 to-emerald-500 rounded-full shadow-lg"></div>
                    <span className="text-gray-300 font-medium">14-day free trial</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <div className="w-3 h-3 bg-gradient-to-r from-emerald-400 to-emerald-500 rounded-full shadow-lg"></div>
                    <span className="text-gray-300 font-medium">Cancel anytime</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  )
}

import Image from "next/image"
import { Target, Users, Lightbulb, ExternalLink, Shield, Code, TrendingUp } from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function About() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      {/* Glass Morphism Hero Section */}
      <section className="relative min-h-screen flex items-center justify-center bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden">
        {/* Animated Background Elements */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse"></div>
          <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/20 to-purple-400/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-emerald-400/20 to-teal-400/20 rounded-full blur-3xl animate-pulse delay-500"></div>
        </div>

        {/* Grid Pattern Overlay */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

        <div className="relative container px-4 py-20">
          <div className="text-center space-y-10 max-w-5xl mx-auto">
            {/* Glass Hero Card */}
            <div className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 lg:p-16 text-white">
              <div className="space-y-8">

                <h1 className="text-4xl lg:text-6xl xl:text-7xl font-bold tracking-tight leading-tight">
                  <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                    About{" "}
                  </span>
                  <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-600 bg-clip-text text-transparent">
                    E-Squared
                  </span>
                  <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                    {" "}Trading
                  </span>
                </h1>

                <p className="text-xl text-gray-200 max-w-4xl mx-auto leading-relaxed">
                  We're on a mission to democratize cryptocurrency trading by making
                  professional-grade strategies accessible to beginners worldwide.
                </p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Glass Morphism Mission Statement */}
      <section className="relative py-20 md:py-32 bg-gradient-to-b from-[#16213e] to-[#0F0C29] dark:from-[#16213e] dark:to-[#0a0a0a] overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/3 left-1/4 w-64 h-64 bg-gradient-to-r from-emerald-500/10 to-teal-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/3 right-1/4 w-80 h-80 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          <div className="grid gap-16 lg:grid-cols-2 lg:gap-20 items-center">
            {/* Mission Content */}
            <div className="space-y-8">

              <h2 className="text-4xl lg:text-5xl font-bold tracking-tight">
                <span className="bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                  Our Mission
                </span>
              </h2>

              <div className="space-y-6">
                <p className="text-lg text-gray-300 leading-relaxed">
                  E-Squared<span className="text-sm align-super">2</span> Trading was founded with a simple belief: everyone deserves
                  access to professional trading tools and strategies, regardless of their
                  experience level.
                </p>
                <p className="text-lg text-gray-300 leading-relaxed">
                  Traditional trading platforms are complex, intimidating, and designed
                  for experts. We're changing that by combining powerful algorithms with
                  intuitive design, making it possible for anyone to build and execute
                  profitable trading strategies.
                </p>
                <p className="text-lg text-gray-300 leading-relaxed">
                  Our proprietary AI analyzes market patterns and guides users through
                  strategy creation, while built-in risk management tools protect their
                  capital. We believe in empowering users with knowledge, not just tools.
                </p>
              </div>
            </div>

            {/* Glass Stats Card */}
            <div className="lg:justify-end flex justify-center">
              <div className="relative">
                {/* Floating orbs */}
                <div className="absolute -top-8 -right-8 w-16 h-16 bg-gradient-to-br from-emerald-400/30 to-teal-500/30 rounded-full blur-xl animate-pulse"></div>
                <div className="absolute -bottom-8 -left-8 w-12 h-12 bg-gradient-to-br from-purple-400/30 to-pink-500/30 rounded-full blur-xl animate-pulse delay-1000"></div>

                {/* Main Stats Card */}
                <div className="border-2 border-[rgba(16,185,129,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.15)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-8 max-w-md text-white">
                  <div className="space-y-8">
                    <div className="flex items-center space-x-4">
                      <div className="h-16 w-16 bg-gradient-to-br from-emerald-500/80 to-teal-600/80 rounded-2xl flex items-center justify-center shadow-lg border border-emerald-400/30">
                        <Target className="h-8 w-8 text-white" />
                      </div>
                      <div>
                        <h3 className="font-bold text-white text-lg">Our Goal</h3>
                        <p className="text-sm text-emerald-100">
                          Make trading accessible to everyone
                        </p>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-6 text-center">
                      <div className="border border-[rgba(99,102,241,0.2)] rounded-xl bg-gradient-to-br from-[rgba(99,102,241,0.1)] to-[rgba(99,102,241,0.02)] backdrop-blur-sm p-4">
                        <div className="text-3xl font-bold text-indigo-300">10K+</div>
                        <div className="text-sm text-gray-300 font-medium">Active Users</div>
                      </div>
                      <div className="border border-[rgba(16,185,129,0.2)] rounded-xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-sm p-4">
                        <div className="text-3xl font-bold text-emerald-300">95%</div>
                        <div className="text-sm text-gray-300 font-medium">Success Rate</div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Glass Morphism Team Section */}
      <section className="relative py-20 md:py-32 bg-gradient-to-b from-[#0F0C29] to-[#24243e] dark:from-[#0a0a0a] dark:to-[#1a1a2e] overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 right-1/3 w-72 h-72 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-blue-500/10 to-purple-500/10 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          {/* Section Header */}
          <div className="text-center space-y-8 mb-20">

            <h2 className="text-4xl lg:text-5xl xl:text-6xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Meet Our
              </span>
              <span className="bg-gradient-to-r from-purple-400 via-pink-400 to-purple-600 bg-clip-text text-transparent">
                {" "}Team
              </span>
            </h2>

            <p className="text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed">
              Our diverse team combines decades of experience in finance, technology,
              and user experience design to revolutionize trading.
            </p>
          </div>

          {/* Team Lead Showcase */}
          <div className="flex justify-center">
            <div className="max-w-4xl w-full">
              <div className="group relative">
                {/* Floating enhancement orbs */}
                <div className="absolute -top-12 -right-12 w-24 h-24 bg-gradient-to-br from-purple-400/30 to-pink-500/30 rounded-full blur-2xl animate-pulse"></div>
                <div className="absolute -bottom-12 -left-12 w-20 h-20 bg-gradient-to-br from-emerald-400/30 to-teal-500/30 rounded-full blur-2xl animate-pulse delay-1000"></div>
                <div className="absolute top-1/2 -right-8 w-16 h-16 bg-gradient-to-br from-blue-400/40 to-purple-500/40 rounded-full blur-xl animate-pulse delay-500"></div>

                <div className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-12 text-white transition-all duration-500 hover:border-[rgba(147,51,234,0.5)] hover:translate-y-2 shadow-2xl">
                  <div className="grid lg:grid-cols-2 gap-12 items-center">
                    {/* Left: Profile Image and Core Info */}
                    <div className="text-center lg:text-left space-y-8">
                      <div className="relative inline-block">
                        <div className="h-32 w-32 bg-gradient-to-br from-purple-500/90 to-pink-600/90 rounded-full flex items-center justify-center mx-auto lg:mx-0 border-4 border-purple-400/30 group-hover:scale-110 transition-transform duration-500 shadow-2xl relative overflow-hidden">
                          <div className="absolute inset-0 bg-gradient-to-br from-purple-400/20 to-transparent rounded-full"></div>
                          <span className="text-3xl font-bold text-white relative z-10">EAK</span>
                        </div>
                        {/* Floating badges */}
                        <div className="absolute -top-2 -right-2 p-2 bg-gradient-to-br from-emerald-500/80 to-teal-600/80 rounded-full border border-emerald-400/30 shadow-lg">
                          <Shield className="h-4 w-4 text-white" />
                        </div>
                        <div className="absolute -bottom-2 -left-2 p-2 bg-gradient-to-br from-blue-500/80 to-purple-600/80 rounded-full border border-blue-400/30 shadow-lg">
                          <Code className="h-4 w-4 text-white" />
                        </div>
                        <div className="absolute top-1/2 -right-4 p-2 bg-gradient-to-br from-pink-500/80 to-rose-600/80 rounded-full border border-pink-400/30 shadow-lg">
                          <TrendingUp className="h-4 w-4 text-white" />
                        </div>
                      </div>

                      <div className="space-y-4">
                        <h3 className="text-3xl lg:text-4xl font-bold text-white">Emmanuel A. Kwofie</h3>
                        <div className="space-y-2">
                          <p className="text-xl text-purple-200 font-semibold">Founder & Lead Developer</p>
                          <div className="flex flex-wrap gap-3 justify-center lg:justify-start">
                            <span className="px-3 py-1 bg-gradient-to-r from-emerald-500/20 to-teal-500/20 border border-emerald-400/30 rounded-full text-sm text-emerald-200 font-medium">Security Researcher</span>
                            <span className="px-3 py-1 bg-gradient-to-r from-blue-500/20 to-purple-500/20 border border-blue-400/30 rounded-full text-sm text-blue-200 font-medium">Software Engineer</span>
                            <span className="px-3 py-1 bg-gradient-to-r from-pink-500/20 to-rose-500/20 border border-pink-400/30 rounded-full text-sm text-pink-200 font-medium">Experienced Trader</span>
                          </div>
                        </div>
                      </div>

                      {/* Social Links */}
                      <div className="flex gap-4 justify-center lg:justify-start">
                        <a
                          href="https://www.linkedin.com/in/eakwofie/"
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-blue-600/80 to-blue-700/80 hover:from-blue-500/90 hover:to-blue-600/90 rounded-xl border border-blue-400/30 backdrop-blur-sm transition-all duration-200 hover:translate-y-0.5 shadow-lg group"
                        >
                          <ExternalLink className="h-4 w-4 text-blue-200 group-hover:text-white transition-colors" />
                          <span className="text-blue-100 group-hover:text-white font-medium transition-colors">LinkedIn</span>
                        </a>
                        <a
                          href="https://www.eakwofie.com/"
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-2 px-6 py-3 bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 rounded-xl border border-purple-400/30 backdrop-blur-sm transition-all duration-200 hover:translate-y-0.5 shadow-lg group"
                        >
                          <ExternalLink className="h-4 w-4 text-purple-200 group-hover:text-white transition-colors" />
                          <span className="text-purple-100 group-hover:text-white font-medium transition-colors">Portfolio</span>
                        </a>
                      </div>
                    </div>

                    {/* Right: Detailed Profile */}
                    <div className="space-y-6">
                      <div className="space-y-4">
                        <h4 className="text-2xl font-bold text-white">About Emmanuel</h4>
                        <p className="text-gray-300 leading-relaxed text-lg">
                          Emmanuel brings a unique combination of deep technical expertise and real-world trading experience to E-Squared Trading. As a seasoned software engineer and security researcher, he understands the critical importance of building secure, scalable systems that traders can trust with their capital.
                        </p>
                        <p className="text-gray-300 leading-relaxed text-lg">
                          With extensive experience in cryptocurrency markets and algorithmic trading strategies, Emmanuel has personally navigated the complexities of trading and understands the challenges beginners face. This firsthand experience drives his passion for creating intuitive tools that democratize access to professional-grade trading strategies.
                        </p>
                      </div>

                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Glass Morphism Vision Section */}
      <section className="relative py-20 md:py-32 bg-gradient-to-b from-[#24243e] to-[#0F0C29] dark:from-[#1a1a2e] dark:to-[#0a0a0a] overflow-hidden">
        {/* Background decoration */}
        <div className="absolute inset-0">
          <div className="absolute top-1/3 left-1/4 w-80 h-80 bg-gradient-to-r from-emerald-500/10 to-teal-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/3 right-1/4 w-64 h-64 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          {/* Section Header */}
          <div className="text-center space-y-8 mb-20">

            <h2 className="text-4xl lg:text-5xl xl:text-6xl font-bold tracking-tight">
              <span className="bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                Our{" "}
              </span>
              <span className="bg-gradient-to-r from-emerald-400 via-teal-400 to-emerald-600 bg-clip-text text-transparent">
                Vision
              </span>
            </h2>

            <p className="text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed">
              We're building the future of trading - where artificial intelligence
              meets human intuition to create unprecedented opportunities.
            </p>
          </div>

          {/* Vision Cards Grid */}
          <div className="grid gap-8 md:grid-cols-3">
            {/* Vision Card 1 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(99,102,241,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(99,102,241,0.1)] to-[rgba(99,102,241,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(99,102,241,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-indigo-500/30 to-purple-500/30 rounded-2xl w-fit backdrop-blur-sm border border-indigo-400/20 group-hover:scale-110 transition-transform duration-300">
                    <Lightbulb className="h-8 w-8 text-indigo-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">AI-Powered Innovation</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Continuously evolving our AI to provide smarter insights,
                    better predictions, and more sophisticated trading strategies.
                  </p>
                </div>
              </div>
            </div>

            {/* Vision Card 2 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(244,63,94,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(244,63,94,0.1)] to-[rgba(244,63,94,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(244,63,94,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-pink-500/30 to-rose-500/30 rounded-2xl w-fit backdrop-blur-sm border border-pink-400/20 group-hover:scale-110 transition-transform duration-300">
                    <Users className="h-8 w-8 text-pink-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">Global Community</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Building a worldwide community of traders who share knowledge,
                    strategies, and support each other's success.
                  </p>
                </div>
              </div>
            </div>

            {/* Vision Card 3 */}
            <div className="group">
              <div className="h-auto border-2 border-[rgba(16,185,129,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-8 text-white transition-all duration-500 hover:border-[rgba(16,185,129,0.4)] hover:translate-y-2">
                <div className="space-y-6">
                  <div className="p-4 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-2xl w-fit backdrop-blur-sm border border-emerald-400/20 group-hover:scale-110 transition-transform duration-300">
                    <Target className="h-8 w-8 text-emerald-200" />
                  </div>

                  <h3 className="text-2xl font-bold text-white">Market Expansion</h3>

                  <p className="text-gray-300 leading-relaxed">
                    Expanding beyond cryptocurrency to include stocks, forex,
                    and commodities trading with the same beginner-friendly approach.
                  </p>
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
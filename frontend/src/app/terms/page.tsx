import Link from "next/link"
import { ArrowRight, TrendingUp, Shield, FileText } from "lucide-react"
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function Terms() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      {/* Terms of Service Content */}
      <section className="relative flex-1 py-20 bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden">
        {/* Animated Background Elements */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/10 to-pink-400/10 rounded-full blur-3xl animate-pulse"></div>
          <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/10 to-purple-400/10 rounded-full blur-3xl animate-pulse delay-1000"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-indigo-400/10 to-blue-400/10 rounded-full blur-3xl animate-pulse delay-500"></div>
        </div>

        {/* Grid Pattern Overlay */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

        <div className="relative container px-4 max-w-4xl mx-auto">
          {/* Page Header */}
          <div className="text-center mb-12">
            <div className="flex items-center justify-center space-x-3 mb-6">
              <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-2xl backdrop-blur-sm border border-purple-400/20">
                <FileText className="h-8 w-8 text-purple-200" />
              </div>
              <h1 className="text-4xl lg:text-5xl font-bold bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Terms of Service
              </h1>
            </div>
            <p className="text-xl text-gray-300 max-w-2xl mx-auto">
              Please read these terms carefully before using E-Squared Tradings platform
            </p>
            <p className="text-sm text-gray-400 mt-4">Last updated: December 15, 2025</p>
          </div>

          {/* Glass Morphism Content Card */}
          <div className="border-2 border-[rgba(147,51,234,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8 lg:p-12 text-white">
            <div className="prose prose-invert max-w-none">

              {/* Section 1 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  1. Acceptance of Terms
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  By accessing and using E-Squared Tradings ("the Platform"), you accept and agree to be bound by the terms and provision of this agreement. If you do not agree to abide by the above, please do not use this service.
                </p>
                <p className="text-gray-300 leading-relaxed">
                  These Terms of Service constitute a legally binding agreement between you and E-Squared Tradings regarding your use of our cryptocurrency trading strategy platform and related services.
                </p>
              </section>

              {/* Section 2 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  2. Service Description
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  E-Squared Tradings provides a platform for creating, testing, and executing cryptocurrency trading strategies. Our services include:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6">
                  <li>• Strategy building tools and templates</li>
                  <li>• Risk management and portfolio optimization</li>
                  <li>• Integration with supported cryptocurrency exchanges</li>
                  <li>• Educational resources and market analysis</li>
                  <li>• Automated trading execution (where available)</li>
                </ul>
              </section>

              {/* Section 3 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  3. User Responsibilities
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  As a user of our platform, you agree to:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6 mb-4">
                  <li>• Provide accurate and complete registration information</li>
                  <li>• Maintain the security of your account credentials</li>
                  <li>• Comply with all applicable laws and regulations</li>
                  <li>• Use the platform only for lawful purposes</li>
                  <li>• Not attempt to manipulate or exploit the platform</li>
                </ul>
                <p className="text-gray-300 leading-relaxed">
                  You are solely responsible for all trading decisions made using our platform. E-Squared Tradings does not provide investment advice.
                </p>
              </section>

              {/* Section 4 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  4. Risk Disclosure
                </h2>
                <div className="p-6 border border-amber-400/30 rounded-xl bg-amber-500/10 backdrop-blur-sm mb-4">
                  <p className="text-amber-300 font-semibold mb-2">⚠️ Important Risk Warning</p>
                  <p className="text-amber-200 leading-relaxed">
                    Cryptocurrency trading involves substantial risk of loss and is not suitable for all investors.
                    Past performance does not guarantee future results. You should carefully consider whether
                    cryptocurrency trading is appropriate for you in light of your experience, objectives,
                    financial resources, and other relevant circumstances.
                  </p>
                </div>
                <p className="text-gray-300 leading-relaxed">
                  By using our platform, you acknowledge that you understand these risks and that any
                  trading decisions are made at your own discretion and risk.
                </p>
              </section>

              {/* Section 5 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  5. Limitation of Liability
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  E-Squared Tradings shall not be liable for any direct, indirect, incidental, special,
                  consequential, or punitive damages, including but not limited to loss of profits,
                  data, or other intangible losses resulting from:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6">
                  <li>• Your use or inability to use the platform</li>
                  <li>• Trading losses incurred through platform use</li>
                  <li>• Unauthorized access to your account</li>
                  <li>• Platform downtime or technical issues</li>
                  <li>• Third-party exchange or service failures</li>
                </ul>
              </section>

              {/* Section 6 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  6. Account Termination
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We reserve the right to terminate or suspend your account immediately, without prior
                  notice or liability, for any reason whatsoever, including without limitation if you
                  breach the Terms.
                </p>
                <p className="text-gray-300 leading-relaxed">
                  Upon termination, your right to use the platform will cease immediately. If you wish
                  to terminate your account, you may simply discontinue using the platform.
                </p>
              </section>

              {/* Section 7 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  7. Governing Law
                </h2>
                <p className="text-gray-300 leading-relaxed">
                  These Terms shall be interpreted and governed by the laws of the jurisdiction in which
                  E-Squared Tradings operates, without regard to its conflict of law provisions.
                  Any disputes arising from these terms will be subject to the exclusive jurisdiction
                  of the courts in that jurisdiction.
                </p>
              </section>

              {/* Section 8 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  8. Changes to Terms
                </h2>
                <p className="text-gray-300 leading-relaxed">
                  We reserve the right, at our sole discretion, to modify or replace these Terms at
                  any time. If a revision is material, we will try to provide at least 30 days notice
                  prior to any new terms taking effect. What constitutes a material change will be
                  determined at our sole discretion.
                </p>
              </section>

              {/* Contact Section */}
              <section className="border-t border-white/20 pt-8">
                <h2 className="text-2xl font-bold text-purple-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-purple-400 rounded-full mr-3"></div>
                  Contact Information
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  If you have any questions about these Terms of Service, please contact us:
                </p>
                <div className="space-y-2 text-gray-300">
                  <p>Email: legal@e-squaredtradings.com</p>
                  <p>Address: [Your Business Address]</p>
                  <p>Phone: [Your Phone Number]</p>
                </div>
              </section>

            </div>
          </div>

          {/* Navigation Links */}
          <div className="flex flex-col sm:flex-row justify-between items-center mt-8 space-y-4 sm:space-y-0">
            <Link
              href="/privacy"
              className="inline-flex items-center space-x-2 text-purple-300 hover:text-purple-200 font-medium transition-colors"
            >
              <Shield className="w-5 h-5" />
              <span>Privacy Policy</span>
              <ArrowRight className="w-4 h-4" />
            </Link>

            <Link
              href="/"
              className="inline-flex items-center space-x-2 text-white hover:text-purple-300 font-medium transition-colors"
            >
              <ArrowRight className="w-5 h-5 rotate-180" />
              <span>Back to Home</span>
            </Link>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  )
}
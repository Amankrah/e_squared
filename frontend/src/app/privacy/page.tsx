import Link from "next/link"
import { ArrowRight, TrendingUp, Shield, Lock, Eye, Database } from "lucide-react"
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function Privacy() {
  return (
    <div className="min-h-screen flex flex-col">
      <Header />

      {/* Privacy Policy Content */}
      <section className="relative flex-1 py-20 bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden">
        {/* Animated Background Elements */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-emerald-400/10 to-teal-400/10 rounded-full blur-3xl animate-pulse"></div>
          <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/10 to-emerald-400/10 rounded-full blur-3xl animate-pulse delay-1000"></div>
          <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-purple-400/10 to-blue-400/10 rounded-full blur-3xl animate-pulse delay-500"></div>
        </div>

        {/* Grid Pattern Overlay */}
        <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

        <div className="relative container px-4 max-w-4xl mx-auto">
          {/* Page Header */}
          <div className="text-center mb-12">
            <div className="flex items-center justify-center space-x-3 mb-6">
              <div className="p-3 bg-gradient-to-br from-emerald-500/30 to-teal-500/30 rounded-2xl backdrop-blur-sm border border-emerald-400/20">
                <Shield className="h-8 w-8 text-emerald-200" />
              </div>
              <h1 className="text-4xl lg:text-5xl font-bold bg-gradient-to-r from-white via-emerald-100 to-emerald-300 bg-clip-text text-transparent">
                Privacy Policy
              </h1>
            </div>
            <p className="text-xl text-gray-300 max-w-2xl mx-auto">
              Your privacy is important to us. Learn how we collect, use, and protect your information.
            </p>
            <p className="text-sm text-gray-400 mt-4">Last updated: December 15, 2025</p>
          </div>

          {/* Glass Morphism Content Card */}
          <div className="border-2 border-[rgba(16,185,129,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl p-8 lg:p-12 text-white">
            <div className="prose prose-invert max-w-none">

              {/* Section 1 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  1. Information We Collect
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We collect information you provide directly to us, such as when you create an account,
                  use our services, or contact us for support.
                </p>

                <div className="space-y-6">
                  <div className="p-4 border border-emerald-400/20 rounded-xl bg-emerald-500/5 backdrop-blur-sm">
                    <h4 className="text-emerald-200 font-semibold mb-2 flex items-center">
                      <Database className="w-4 h-4 mr-2" />
                      Personal Information
                    </h4>
                    <ul className="text-gray-300 leading-relaxed space-y-1 ml-6">
                      <li>• Name and email address</li>
                      <li>• Account credentials and security information</li>
                      <li>• Profile information and preferences</li>
                      <li>• Communication history with our support team</li>
                    </ul>
                  </div>

                  <div className="p-4 border border-blue-400/20 rounded-xl bg-blue-500/5 backdrop-blur-sm">
                    <h4 className="text-blue-200 font-semibold mb-2 flex items-center">
                      <Eye className="w-4 h-4 mr-2" />
                      Usage Information
                    </h4>
                    <ul className="text-gray-300 leading-relaxed space-y-1 ml-6">
                      <li>• Trading strategies and configurations</li>
                      <li>• Platform usage patterns and features accessed</li>
                      <li>• Device information and IP addresses</li>
                      <li>• Performance and error logs</li>
                    </ul>
                  </div>

                  <div className="p-4 border border-purple-400/20 rounded-xl bg-purple-500/5 backdrop-blur-sm">
                    <h4 className="text-purple-200 font-semibold mb-2 flex items-center">
                      <Lock className="w-4 h-4 mr-2" />
                      Financial Information
                    </h4>
                    <ul className="text-gray-300 leading-relaxed space-y-1 ml-6">
                      <li>• Exchange API keys (encrypted and secure)</li>
                      <li>• Trading history and performance data</li>
                      <li>• Payment information for subscription services</li>
                      <li>• Portfolio and balance information</li>
                    </ul>
                  </div>
                </div>
              </section>

              {/* Section 2 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  2. How We Use Your Information
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We use the information we collect to provide, maintain, and improve our services:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6 mb-4">
                  <li>• Provide and operate the E-Squared Tradings platform</li>
                  <li>• Execute your trading strategies and manage your account</li>
                  <li>• Send important updates and security notifications</li>
                  <li>• Improve our services and develop new features</li>
                  <li>• Provide customer support and respond to inquiries</li>
                  <li>• Detect and prevent fraud, abuse, and security incidents</li>
                  <li>• Comply with legal obligations and enforce our terms</li>
                </ul>
              </section>

              {/* Section 3 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  3. Information Sharing and Disclosure
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We do not sell, trade, or otherwise transfer your personal information to third parties,
                  except in the following circumstances:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6 mb-4">
                  <li>• With your explicit consent</li>
                  <li>• To authorized service providers who assist in our operations</li>
                  <li>• To comply with legal requirements or protect our rights</li>
                  <li>• In connection with a business transfer or acquisition</li>
                  <li>• To cryptocurrency exchanges for trade execution (API keys only)</li>
                </ul>

                <div className="p-6 border border-emerald-400/30 rounded-xl bg-emerald-500/10 backdrop-blur-sm">
                  <p className="text-emerald-300 font-semibold mb-2">🔒 Your API Keys Are Secure</p>
                  <p className="text-emerald-200 leading-relaxed">
                    Exchange API keys are encrypted with bank-level security and are only used to execute
                    your trading strategies. We never access your funds or share your keys with third parties.
                  </p>
                </div>
              </section>

              {/* Section 4 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  4. Data Security
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We implement appropriate technical and organizational measures to protect your
                  personal information against unauthorized access, alteration, disclosure, or destruction:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6">
                  <li>• End-to-end encryption for sensitive data</li>
                  <li>• Regular security audits and penetration testing</li>
                  <li>• Secure servers with advanced firewall protection</li>
                  <li>• Multi-factor authentication and access controls</li>
                  <li>• Regular backup and disaster recovery procedures</li>
                </ul>
              </section>

              {/* Section 5 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  5. Cookies and Tracking Technologies
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We use cookies and similar tracking technologies to enhance your experience:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6 mb-4">
                  <li>• Essential cookies for platform functionality</li>
                  <li>• Performance cookies to improve user experience</li>
                  <li>• Analytics cookies to understand usage patterns</li>
                  <li>• Preference cookies to remember your settings</li>
                </ul>
                <p className="text-gray-300 leading-relaxed">
                  You can control cookie settings through your browser preferences, though disabling
                  certain cookies may limit platform functionality.
                </p>
              </section>

              {/* Section 6 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  6. Your Rights and Choices
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  You have certain rights regarding your personal information:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6 mb-4">
                  <li>• Access and review your personal information</li>
                  <li>• Update or correct inaccurate information</li>
                  <li>• Delete your account and associated data</li>
                  <li>• Opt-out of non-essential communications</li>
                  <li>• Request data portability where applicable</li>
                  <li>• Object to certain processing activities</li>
                </ul>
                <p className="text-gray-300 leading-relaxed">
                  To exercise these rights, please contact us using the information provided below.
                </p>
              </section>

              {/* Section 7 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  7. Data Retention
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  We retain your information for as long as necessary to provide our services and
                  comply with legal obligations:
                </p>
                <ul className="text-gray-300 leading-relaxed space-y-2 ml-6">
                  <li>• Account information: Retained while your account is active</li>
                  <li>• Trading data: Retained for 7 years for regulatory compliance</li>
                  <li>• Communication records: Retained for 3 years</li>
                  <li>• Analytics data: Anonymized and retained for service improvement</li>
                </ul>
              </section>

              {/* Section 8 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  8. International Data Transfers
                </h2>
                <p className="text-gray-300 leading-relaxed">
                  Your information may be transferred to and processed in countries other than your own.
                  We ensure appropriate safeguards are in place to protect your information in accordance
                  with this privacy policy and applicable data protection laws.
                </p>
              </section>

              {/* Section 9 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  9. Children's Privacy
                </h2>
                <p className="text-gray-300 leading-relaxed">
                  Our services are not intended for individuals under the age of 18. We do not knowingly
                  collect personal information from children under 18. If we become aware that we have
                  collected personal information from a child under 18, we will delete such information promptly.
                </p>
              </section>

              {/* Section 10 */}
              <section className="mb-10">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  10. Changes to This Policy
                </h2>
                <p className="text-gray-300 leading-relaxed">
                  We may update this privacy policy from time to time. We will notify you of any material
                  changes by posting the new privacy policy on this page and updating the "Last updated" date.
                  We encourage you to review this policy periodically.
                </p>
              </section>

              {/* Contact Section */}
              <section className="border-t border-white/20 pt-8">
                <h2 className="text-2xl font-bold text-emerald-200 mb-4 flex items-center">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full mr-3"></div>
                  Contact Us
                </h2>
                <p className="text-gray-300 leading-relaxed mb-4">
                  If you have any questions about this Privacy Policy or our data practices, please contact us:
                </p>
                <div className="space-y-2 text-gray-300">
                  <p>Email: privacy@e-squaredtradings.com</p>
                  <p>Data Protection Officer: dpo@e-squaredtradings.com</p>
                  <p>Address: [Your Business Address]</p>
                  <p>Phone: [Your Phone Number]</p>
                </div>
              </section>

            </div>
          </div>

          {/* Navigation Links */}
          <div className="flex flex-col sm:flex-row justify-between items-center mt-8 space-y-4 sm:space-y-0">
            <Link
              href="/terms"
              className="inline-flex items-center space-x-2 text-purple-300 hover:text-purple-200 font-medium transition-colors"
            >
              <Shield className="w-5 h-5" />
              <span>Terms of Service</span>
              <ArrowRight className="w-4 h-4" />
            </Link>

            <Link
              href="/"
              className="inline-flex items-center space-x-2 text-white hover:text-emerald-300 font-medium transition-colors"
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
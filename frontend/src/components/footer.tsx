import Link from "next/link"
import { MessageCircle, Twitter, TrendingUp, Mail, Phone } from "lucide-react"
import { ProgrammableLogo } from "@/components/programmable-logo"

export function Footer() {
  return (
    <footer className="relative border-t border-[rgba(147,51,234,0.2)] bg-gradient-to-b from-[#24243e] via-[#302b63] to-[#0F0C29] overflow-hidden">
      {/* Background decoration */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-gradient-to-r from-purple-500/5 to-pink-500/5 rounded-full blur-3xl"></div>
        <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-gradient-to-r from-blue-500/5 to-purple-500/5 rounded-full blur-3xl"></div>
      </div>

      <div className="relative container px-4 py-16">
        <div className="grid grid-cols-1 gap-12 md:grid-cols-4">
          {/* Enhanced Logo and Description */}
          <div className="md:col-span-2">
            <Link href="/" className="mb-6">
              <ProgrammableLogo size="sm" showText={true} />
            </Link>

            <p className="text-gray-300 max-w-md leading-relaxed mb-6">
              Empowering beginners to trade cryptocurrency like professionals with
              our easy-to-use strategy builder and risk management tools.
            </p>

            {/* Contact info cards */}
            <div className="space-y-3">
              <div className="flex items-center space-x-3 border border-[rgba(16,185,129,0.2)] rounded-xl bg-gradient-to-r from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-sm p-3">
                <div className="p-2 bg-emerald-500/20 rounded-lg">
                  <Mail className="h-4 w-4 text-emerald-300" />
                </div>
                <span className="text-emerald-100 text-sm font-medium">contact@esquaredtradings.com</span>
              </div>
            </div>
          </div>

          {/* Enhanced Quick Links */}
          <div>
            <h3 className="font-bold text-white mb-6 text-lg">Quick Links</h3>
            <ul className="space-y-3">
              <li>
                <Link href="/about" className="relative group inline-flex items-center text-gray-300 hover:text-white transition-all duration-300">
                  <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40 -m-2 p-2"></div>
                  <span className="relative">About Us</span>
                </Link>
              </li>
              <li>
                <Link href="/contact" className="relative group inline-flex items-center text-gray-300 hover:text-white transition-all duration-300">
                  <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40 -m-2 p-2"></div>
                  <span className="relative">Contact</span>
                </Link>
              </li>
              <li>
                <Link href="/dashboard" className="relative group inline-flex items-center text-gray-300 hover:text-white transition-all duration-300">
                  <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40 -m-2 p-2"></div>
                  <span className="relative">Dashboard</span>
                </Link>
              </li>
              <li>
                <Link href="/privacy" className="relative group inline-flex items-center text-gray-300 hover:text-white transition-all duration-300">
                  <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40 -m-2 p-2"></div>
                  <span className="relative">Privacy Policy</span>
                </Link>
              </li>
              <li>
                <Link href="/terms" className="relative group inline-flex items-center text-gray-300 hover:text-white transition-all duration-300">
                  <div className="absolute inset-0 rounded-lg bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40 -m-2 p-2"></div>
                  <span className="relative">Terms of Service</span>
                </Link>
              </li>
            </ul>
          </div>

          {/* Enhanced Social Links */}
          <div>
            <h3 className="font-bold text-white mb-6 text-lg">Connect With Us</h3>

            {/* Social buttons */}
            <div className="flex flex-wrap gap-3 mb-6">
              <Link
                href="https://x.com/esquaredtrading"
                className="group relative"
                target="_blank"
                rel="noopener noreferrer"
              >
                <div className="h-12 w-12 border border-[rgba(29,161,242,0.3)] rounded-xl bg-gradient-to-br from-[rgba(29,161,242,0.1)] to-[rgba(29,161,242,0.02)] backdrop-blur-xl flex items-center justify-center group-hover:border-[rgba(29,161,242,0.5)] transition-all duration-300 group-hover:scale-110">
                  <Twitter className="h-5 w-5 text-blue-300 group-hover:text-blue-200" />
                </div>
              </Link>

              <Link
                href="https://t.me/esquaredtrading"
                className="group relative"
                target="_blank"
                rel="noopener noreferrer"
              >
                <div className="h-12 w-12 border border-[rgba(34,139,230,0.3)] rounded-xl bg-gradient-to-br from-[rgba(34,139,230,0.1)] to-[rgba(34,139,230,0.02)] backdrop-blur-xl flex items-center justify-center group-hover:border-[rgba(34,139,230,0.5)] transition-all duration-300 group-hover:scale-110">
                  <MessageCircle className="h-5 w-5 text-blue-300 group-hover:text-blue-200" />
                </div>
              </Link>

              <Link
                href="https://discord.gg/esquaredtrading"
                className="group relative"
                target="_blank"
                rel="noopener noreferrer"
              >
                <div className="h-12 w-12 border border-[rgba(114,137,218,0.3)] rounded-xl bg-gradient-to-br from-[rgba(114,137,218,0.1)] to-[rgba(114,137,218,0.02)] backdrop-blur-xl flex items-center justify-center group-hover:border-[rgba(114,137,218,0.5)] transition-all duration-300 group-hover:scale-110">
                  <MessageCircle className="h-5 w-5 text-indigo-300 group-hover:text-indigo-200" />
                </div>
              </Link>
            </div>

            {/* Newsletter signup */}
            <div className="border border-[rgba(147,51,234,0.2)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-sm p-4">
              <h4 className="font-semibold text-white mb-2 text-sm">Stay Updated</h4>
              <p className="text-gray-300 text-xs mb-3">Get trading tips and platform updates</p>
              <button className="w-full h-9 px-4 border border-purple-400/50 rounded-lg bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white text-sm font-medium transition-all duration-300 backdrop-blur-sm">
                Subscribe
              </button>
            </div>
          </div>
        </div>

        {/* Enhanced Copyright */}
        <div className="mt-16 pt-8 border-t border-[rgba(147,51,234,0.2)]">
          <div className="flex flex-col md:flex-row justify-between items-center gap-4">
            <p className="text-gray-400 text-sm">
              © {new Date().getFullYear()} E-Squared Trading. All rights reserved.
            </p>

            <div className="flex items-center space-x-6 text-xs text-gray-400">
              <span>Built by traders for traders</span>
              <div className="w-2 h-2 bg-gradient-to-r from-emerald-400 to-emerald-500 rounded-full animate-pulse"></div>
              <span>Platform Online</span>
            </div>
          </div>
        </div>
      </div>
    </footer>
  )
}
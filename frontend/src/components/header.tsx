"use client"

import Link from "next/link"
import { useState } from "react"
import { Menu, X } from "lucide-react"
import { ThemeToggle } from "@/components/theme-toggle"
import { ProgrammableLogo } from "@/components/programmable-logo"
import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
} from "@/components/ui/navigation-menu"

export function Header() {
  const [isMenuOpen, setIsMenuOpen] = useState(false)

  const toggleMenu = () => setIsMenuOpen(!isMenuOpen)

  return (
    <header className="sticky top-0 z-50 w-full border-b border-[rgba(147,51,234,0.2)] bg-gradient-to-r from-[rgba(15,12,41,0.95)] via-[rgba(36,36,62,0.95)] to-[rgba(48,43,99,0.95)] backdrop-blur-xl">
      <div className="container flex h-20 items-center justify-between px-4">
        {/* Programmable E-Squared Logo */}
        <Link href="/">
          <ProgrammableLogo size="md" showText={true} />
        </Link>

        {/* Glass Morphism Navigation */}
        <nav className="hidden md:flex items-center space-x-2">
          <NavigationMenu>
            <NavigationMenuList className="space-x-2">
              <NavigationMenuItem>
                <NavigationMenuLink asChild>
                  <Link
                    href="/"
                    className="relative group inline-flex h-10 w-max items-center justify-center rounded-xl px-5 py-2 text-sm font-medium transition-all duration-300 text-gray-200 hover:text-white focus:outline-none"
                  >
                    <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40"></div>
                    <span className="relative">Home</span>
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink asChild>
                  <Link
                    href="/about"
                    className="relative group inline-flex h-10 w-max items-center justify-center rounded-xl px-5 py-2 text-sm font-medium transition-all duration-300 text-gray-200 hover:text-white focus:outline-none"
                  >
                    <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40"></div>
                    <span className="relative">About</span>
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink asChild>
                  <Link
                    href="/contact"
                    className="relative group inline-flex h-10 w-max items-center justify-center rounded-xl px-5 py-2 text-sm font-medium transition-all duration-300 text-gray-200 hover:text-white focus:outline-none"
                  >
                    <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40"></div>
                    <span className="relative">Contact</span>
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink asChild>
                  <Link
                    href="/dashboard"
                    className="relative group inline-flex h-10 w-max items-center justify-center rounded-xl px-5 py-2 text-sm font-medium transition-all duration-300 text-gray-200 hover:text-white focus:outline-none"
                  >
                    <div className="absolute inset-0 rounded-xl bg-gradient-to-r from-purple-500/10 to-pink-500/10 opacity-0 group-hover:opacity-100 transition-opacity duration-300 backdrop-blur-sm border border-purple-400/20 group-hover:border-purple-400/40"></div>
                    <span className="relative">Dashboard</span>
                  </Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
        </nav>

        {/* Enhanced Glass Morphism Auth & Theme */}
        <div className="hidden md:flex items-center space-x-3">
          <div className="border border-[rgba(147,51,234,0.2)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-sm p-1">
            <ThemeToggle />
          </div>

          <button className="h-10 px-5 border border-white/20 rounded-xl bg-white/10 hover:bg-white/20 text-gray-200 hover:text-white font-medium transition-all duration-300 backdrop-blur-sm">
            <Link href="/login">Login</Link>
          </button>

          <button className="h-10 px-5 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white font-medium shadow-lg hover:shadow-xl transition-all duration-300 backdrop-blur-sm hover:translate-y-0.5">
            <Link href="/signup">Sign Up</Link>
          </button>
        </div>

        {/* Glass Morphism Mobile Menu Button */}
        <div className="flex items-center space-x-3 md:hidden">
          <div className="border border-[rgba(147,51,234,0.2)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-sm p-1">
            <ThemeToggle />
          </div>

          <button
            onClick={toggleMenu}
            className="h-10 w-10 border border-[rgba(147,51,234,0.3)] rounded-xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.05)] backdrop-blur-sm flex items-center justify-center text-gray-200 hover:text-white transition-all duration-300 hover:border-[rgba(147,51,234,0.5)]"
            aria-label="Toggle menu"
          >
            {isMenuOpen ? <X className="h-5 w-5" /> : <Menu className="h-5 w-5" />}
          </button>
        </div>
      </div>

      {/* Enhanced Glass Morphism Mobile Navigation */}
      {isMenuOpen && (
        <div className="border-t border-[rgba(147,51,234,0.2)] bg-gradient-to-b from-[rgba(15,12,41,0.98)] to-[rgba(36,36,62,0.98)] backdrop-blur-xl md:hidden">
          <div className="container px-4 py-8">
            <nav className="flex flex-col space-y-3">
              <Link
                href="/"
                className="block px-6 py-4 text-lg font-medium transition-all duration-300 text-gray-200 hover:text-white rounded-xl border border-transparent hover:border-[rgba(147,51,234,0.3)] hover:bg-gradient-to-r hover:from-[rgba(147,51,234,0.1)] hover:to-[rgba(147,51,234,0.05)] backdrop-blur-sm"
                onClick={toggleMenu}
              >
                Home
              </Link>
              <Link
                href="/about"
                className="block px-6 py-4 text-lg font-medium transition-all duration-300 text-gray-200 hover:text-white rounded-xl border border-transparent hover:border-[rgba(147,51,234,0.3)] hover:bg-gradient-to-r hover:from-[rgba(147,51,234,0.1)] hover:to-[rgba(147,51,234,0.05)] backdrop-blur-sm"
                onClick={toggleMenu}
              >
                About
              </Link>
              <Link
                href="/contact"
                className="block px-6 py-4 text-lg font-medium transition-all duration-300 text-gray-200 hover:text-white rounded-xl border border-transparent hover:border-[rgba(147,51,234,0.3)] hover:bg-gradient-to-r hover:from-[rgba(147,51,234,0.1)] hover:to-[rgba(147,51,234,0.05)] backdrop-blur-sm"
                onClick={toggleMenu}
              >
                Contact
              </Link>
              <Link
                href="/dashboard"
                className="block px-6 py-4 text-lg font-medium transition-all duration-300 text-gray-200 hover:text-white rounded-xl border border-transparent hover:border-[rgba(147,51,234,0.3)] hover:bg-gradient-to-r hover:from-[rgba(147,51,234,0.1)] hover:to-[rgba(147,51,234,0.05)] backdrop-blur-sm"
                onClick={toggleMenu}
              >
                Dashboard
              </Link>

              <div className="flex flex-col space-y-4 pt-8">
                <button className="px-6 py-4 text-lg font-medium text-gray-200 hover:text-white rounded-xl border border-white/20 bg-white/10 hover:bg-white/20 transition-all duration-300 backdrop-blur-sm">
                  <Link href="/login" onClick={toggleMenu}>Login</Link>
                </button>

                <button className="px-6 py-4 text-lg font-medium text-white rounded-xl border border-purple-400/50 bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 shadow-lg hover:shadow-xl transition-all duration-300 backdrop-blur-sm">
                  <Link href="/signup" onClick={toggleMenu}>Sign Up</Link>
                </button>
              </div>
            </nav>
          </div>
        </div>
      )}
    </header>
  )
}
"use client"

import Link from "next/link"
import { useState } from "react"
import { Eye, EyeOff, ArrowRight, Shield, TrendingUp } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useAuth } from "@/contexts/auth-context"

export default function SignUp() {
  const [showPassword, setShowPassword] = useState(false)
  const [showConfirmPassword, setShowConfirmPassword] = useState(false)
  const [formData, setFormData] = useState({
    email: "",
    password: "",
    confirmPassword: "",
    agreeToTerms: false
  })

  const { signup, isLoading, error, clearError } = useAuth()

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target
    setFormData(prev => ({
      ...prev,
      [name]: type === "checkbox" ? checked : value
    }))
    // Clear error when user starts typing
    if (error) {
      clearError()
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    // Validate password confirmation
    if (formData.password !== formData.confirmPassword) {
      // This should be handled by setting an error state
      return
    }

    try {
      await signup({
        email: formData.email,
        password: formData.password,
      })
    } catch (error) {
      // Error is handled by the auth context
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] dark:from-[#0a0a0a] dark:via-[#1a1a2e] dark:to-[#16213e] overflow-hidden relative">
      {/* Animated Background Elements */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 left-1/4 w-72 h-72 bg-gradient-to-r from-purple-400/20 to-pink-400/20 rounded-full blur-3xl animate-pulse"></div>
        <div className="absolute top-3/4 right-1/4 w-96 h-96 bg-gradient-to-r from-blue-400/20 to-purple-400/20 rounded-full blur-3xl animate-pulse delay-1000"></div>
        <div className="absolute bottom-1/4 left-1/3 w-64 h-64 bg-gradient-to-r from-indigo-400/20 to-blue-400/20 rounded-full blur-3xl animate-pulse delay-500"></div>
      </div>

      {/* Grid Pattern Overlay */}
      <div className="absolute inset-0 bg-[url('data:image/svg+xml,%3Csvg%20width%3D%2240%22%20height%3D%2240%22%20viewBox%3D%220%200%2040%2040%22%20xmlns%3D%22http%3A//www.w3.org/2000/svg%22%3E%3Cg%20fill%3D%22%23ffffff%22%20fill-opacity%3D%220.02%22%3E%3Cpath%20d%3D%22M0%200h40v40H0V0z%22/%3E%3C/g%3E%3C/svg%3E')] opacity-50"></div>

      <div className="relative w-full max-w-lg mx-auto p-6">
        {/* Logo and Back Link */}
        <div className="text-center mb-8">
          <Link href="/" className="inline-flex items-center space-x-2 text-white hover:text-purple-300 transition-colors mb-6">
            <ArrowRight className="w-5 h-5 rotate-180" />
            <span className="text-lg font-medium">Back to Home</span>
          </Link>

          <div className="flex items-center justify-center space-x-3 mb-2">
            <div className="p-3 bg-gradient-to-br from-purple-500/30 to-pink-500/30 rounded-2xl backdrop-blur-sm border border-purple-400/20">
              <TrendingUp className="h-8 w-8 text-purple-200" />
            </div>
            <h1 className="text-3xl font-bold bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
              E-Squared
            </h1>
          </div>
          <p className="text-gray-300 text-lg">Join thousands of successful traders</p>
        </div>

        {/* Glass Morphism Signup Card */}
        <div className="border-2 border-[rgba(147,51,234,0.3)] rounded-3xl bg-gradient-to-br from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl p-8">
          <div className="space-y-6">
            <div className="text-center space-y-2">
              <h2 className="text-2xl font-bold text-white">Create Your Account</h2>
              <p className="text-gray-300">Start your trading journey today</p>
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Error Message */}
              {error && (
                <div className="p-4 border border-red-400/30 rounded-xl bg-red-500/10 backdrop-blur-sm">
                  <p className="text-red-300 text-sm">{error}</p>
                </div>
              )}

              {/* Password Mismatch Error */}
              {formData.password !== formData.confirmPassword && formData.confirmPassword && (
                <div className="p-4 border border-yellow-400/30 rounded-xl bg-yellow-500/10 backdrop-blur-sm">
                  <p className="text-yellow-300 text-sm">Passwords do not match</p>
                </div>
              )}

              {/* Email Field */}
              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                <Input
                  id="email"
                  name="email"
                  type="email"
                  placeholder="Enter your email"
                  value={formData.email}
                  onChange={handleInputChange}
                  required
                  className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300"
                />
              </div>

              {/* Password Field */}
              <div className="space-y-2">
                <Label htmlFor="password" className="text-gray-200 font-medium">Password</Label>
                <div className="relative">
                  <Input
                    id="password"
                    name="password"
                    type={showPassword ? "text" : "password"}
                    placeholder="Create a strong password"
                    value={formData.password}
                    onChange={handleInputChange}
                    required
                    className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300 pr-12"
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors"
                  >
                    {showPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                  </button>
                </div>
              </div>

              {/* Confirm Password Field */}
              <div className="space-y-2">
                <Label htmlFor="confirmPassword" className="text-gray-200 font-medium">Confirm Password</Label>
                <div className="relative">
                  <Input
                    id="confirmPassword"
                    name="confirmPassword"
                    type={showConfirmPassword ? "text" : "password"}
                    placeholder="Confirm your password"
                    value={formData.confirmPassword}
                    onChange={handleInputChange}
                    required
                    className="h-12 border-2 border-white/20 rounded-xl bg-white/10 backdrop-blur-sm placeholder:text-gray-400 text-white focus:border-purple-400/50 focus:bg-white/15 transition-all duration-300 pr-12"
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-white transition-colors"
                  >
                    {showConfirmPassword ? <EyeOff className="h-5 w-5" /> : <Eye className="h-5 w-5" />}
                  </button>
                </div>
              </div>

              {/* Terms Checkbox */}
              <div className="flex items-start space-x-3">
                <input
                  id="agreeToTerms"
                  name="agreeToTerms"
                  type="checkbox"
                  checked={formData.agreeToTerms}
                  onChange={handleInputChange}
                  required
                  className="mt-1 h-4 w-4 rounded border-2 border-white/20 bg-white/10 text-purple-600 focus:ring-purple-500 focus:ring-offset-0"
                />
                <Label htmlFor="agreeToTerms" className="text-sm text-gray-300 leading-relaxed">
                  I agree to the{" "}
                  <Link href="/terms" className="text-purple-300 hover:text-purple-200 underline">
                    Terms of Service
                  </Link>{" "}
                  and{" "}
                  <Link href="/privacy" className="text-purple-300 hover:text-purple-200 underline">
                    Privacy Policy
                  </Link>
                </Label>
              </div>

              {/* Submit Button */}
              <Button
                type="submit"
                disabled={isLoading || formData.password !== formData.confirmPassword}
                className="w-full h-12 border border-purple-400/50 rounded-xl bg-gradient-to-r from-purple-600/90 to-pink-600/90 hover:from-purple-500/95 hover:to-pink-500/95 text-white font-semibold text-lg transition-all duration-300 backdrop-blur-sm hover:translate-y-0.5 shadow-lg hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isLoading ? (
                  <>
                    <div className="w-5 h-5 border-2 border-white/20 border-t-white rounded-full animate-spin mr-2"></div>
                    Creating Account...
                  </>
                ) : (
                  <>
                    Create Account
                    <ArrowRight className="w-5 h-5 ml-2" />
                  </>
                )}
              </Button>
            </form>

            {/* Divider */}
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-white/20"></div>
              </div>
              <div className="relative flex justify-center text-sm">
                <span className="px-4 bg-gradient-to-r from-[rgba(147,51,234,0.15)] to-[rgba(147,51,234,0.02)] text-gray-300">
                  Already have an account?
                </span>
              </div>
            </div>

            {/* Login Link */}
            <div className="text-center">
              <Link
                href="/login"
                className="inline-flex items-center space-x-2 text-purple-300 hover:text-purple-200 font-medium transition-colors"
              >
                <span>Sign in to your account</span>
                <ArrowRight className="w-4 h-4" />
              </Link>
            </div>
          </div>
        </div>

        {/* Trust Indicators */}
        <div className="flex justify-center items-center space-x-6 mt-8 text-sm">
          <div className="flex items-center space-x-2">
            <Shield className="w-4 h-4 text-emerald-400" />
            <span className="text-gray-300">Bank-level Security</span>
          </div>
          <div className="flex items-center space-x-2">
            <div className="w-2 h-2 bg-emerald-400 rounded-full"></div>
            <span className="text-gray-300">SSL Encrypted</span>
          </div>
        </div>
      </div>
    </div>
  )
}
"use client"

import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react'
import { useRouter } from 'next/navigation'
import { apiClient, User, UserProfile, LoginRequest, SignupRequest, ApiError } from '@/lib/api'

interface AuthContextType {
  user: User | null
  profile: UserProfile | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (credentials: LoginRequest) => Promise<void>
  signup: (credentials: SignupRequest) => Promise<void>
  logout: () => void
  refreshUser: () => Promise<void>
  refreshProfile: () => Promise<void>
  error: string | null
  clearError: () => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

interface AuthProviderProps {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null)
  const [profile, setProfile] = useState<UserProfile | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const router = useRouter()

  const clearError = () => setError(null)

  const refreshUser = async () => {
    try {
      const userData = await apiClient.getCurrentUser()
      setUser(userData)
    } catch (error) {
      console.error('Failed to fetch user:', error)
      if (error instanceof ApiError && error.status === 401) {
        // Token is invalid, clear it
        apiClient.logout()
        setUser(null)
        setProfile(null)
      }
    }
  }

  const refreshProfile = async () => {
    try {
      const profileData = await apiClient.getProfile()
      setProfile(profileData)
    } catch (error) {
      // Profile doesn't exist yet - this is normal for new users
      if (error instanceof ApiError && error.status === 404) {
        console.log('User profile not found - user needs to create profile')
        setProfile(null)
      } else {
        console.error('Failed to fetch profile:', error)
        setError('Failed to load profile')
      }
    }
  }

  const login = async (credentials: LoginRequest) => {
    try {
      setIsLoading(true)
      setError(null)
      const response = await apiClient.login(credentials)
      setUser(response.user)

      // Try to load profile
      await refreshProfile()

      router.push('/dashboard')
    } catch (error) {
      if (error instanceof ApiError) {
        setError(error.message)
      } else {
        setError('Login failed. Please try again.')
      }
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const signup = async (credentials: SignupRequest) => {
    try {
      setIsLoading(true)
      setError(null)
      const response = await apiClient.signup(credentials)
      setUser(response.user)

      // Profile won't exist yet for new users
      setProfile(null)

      router.push('/dashboard')
    } catch (error) {
      if (error instanceof ApiError) {
        setError(error.message)
      } else {
        setError('Signup failed. Please try again.')
      }
      throw error
    } finally {
      setIsLoading(false)
    }
  }

  const logout = async () => {
    try {
      await apiClient.logout()
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      setUser(null)
      setProfile(null)
      setError(null)
      router.push('/')
    }
  }

  useEffect(() => {
    const initializeAuth = async () => {
      try {
        // First try to get CSRF token
        await apiClient.getCsrfTokenFromServer()
        // Then try to get current user - this will work if we have a valid cookie
        await refreshUser()
        await refreshProfile()
      } catch (error) {
        // No valid session - user is not authenticated, which is fine
        console.log('No valid session found:', error)
      } finally {
        setIsLoading(false)
      }
    }

    initializeAuth()
  }, [])

  const value: AuthContextType = {
    user,
    profile,
    isLoading,
    isAuthenticated: !!user,
    login,
    signup,
    logout,
    refreshUser,
    refreshProfile,
    error,
    clearError,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}
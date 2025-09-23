"use client"

import { useState, useEffect } from "react"
import { User, Camera, Mail, Phone, MapPin, Calendar, Save, Edit3, Plus } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useAuth } from "@/contexts/auth-context"
import { apiClient } from "@/lib/api"

export function ProfileSettings() {
  const { user, profile, refreshProfile } = useAuth()
  const [isEditing, setIsEditing] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    phone: "",
    location: "",
    bio: ""
  })

  useEffect(() => {
    if (profile) {
      setFormData({
        name: profile.name || "",
        email: profile.email || user?.email || "",
        phone: profile.phone || "",
        location: profile.location || "",
        bio: profile.bio || ""
      })
    } else if (user) {
      setFormData({
        name: "",
        email: user.email,
        phone: "",
        location: "",
        bio: ""
      })
    }
  }, [profile, user])

  const handleSave = async () => {
    if (!user) return

    setIsLoading(true)
    setError(null)

    try {
      if (profile) {
        // Update existing profile
        await apiClient.updateProfile({
          name: formData.name,
          email: formData.email,
          phone: formData.phone || undefined,
          location: formData.location || undefined,
          bio: formData.bio || undefined
        })
      } else {
        // Create new profile
        await apiClient.createProfile({
          name: formData.name,
          email: formData.email,
          phone: formData.phone || undefined,
          location: formData.location || undefined,
          bio: formData.bio || undefined
        })
      }

      await refreshProfile()
      setIsEditing(false)
    } catch (err: any) {
      setError(err.message || 'Failed to save profile')
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* Profile Header */}
      <div className="border-2 border-[rgba(59,130,246,0.2)] rounded-3xl bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl p-8">
        <div className="flex flex-col md:flex-row items-start md:items-center space-y-6 md:space-y-0 md:space-x-8">
          {/* Profile Picture */}
          <div className="relative group">
            <div className="w-32 h-32 bg-gradient-to-r from-purple-500 to-pink-500 rounded-3xl flex items-center justify-center shadow-2xl">
              <User className="w-16 h-16 text-white" />
            </div>
            <button className="absolute inset-0 bg-black/50 rounded-3xl flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
              <Camera className="w-8 h-8 text-white" />
            </button>
          </div>

          {/* Profile Info */}
          <div className="flex-1 space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-3xl font-bold text-white">{profile?.name || formData.name || "User"}</h2>
                <p className="text-blue-300 font-medium">
                  {profile?.is_verified ? "Verified Trader" : "Pending Verification"}
                </p>
              </div>
              <Button
                onClick={() => setIsEditing(!isEditing)}
                className="bg-gradient-to-r from-blue-600/80 to-cyan-600/80 hover:from-blue-500/90 hover:to-cyan-500/90 text-white"
              >
                <Edit3 className="w-4 h-4 mr-2" />
                {isEditing ? "Cancel" : "Edit Profile"}
              </Button>
            </div>

            <div className="grid md:grid-cols-2 gap-4">
              <div className="flex items-center space-x-3 p-3 bg-blue-500/10 rounded-xl border border-blue-400/20">
                <Mail className="w-5 h-5 text-blue-300" />
                <span className="text-blue-200">{profile?.email || user?.email}</span>
              </div>
              <div className="flex items-center space-x-3 p-3 bg-blue-500/10 rounded-xl border border-blue-400/20">
                <Calendar className="w-5 h-5 text-blue-300" />
                <span className="text-blue-200">Joined {profile?.join_date || new Date().toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Profile Form */}
      <Card className="border-2 border-[rgba(59,130,246,0.2)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl">
        <CardHeader>
          <CardTitle className="text-xl font-bold text-white">Personal Information</CardTitle>
          <CardDescription className="text-blue-300">
            Update your personal details and contact information
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-400/20 rounded-xl text-red-300">
              {error}
            </div>
          )}

          {!profile && (
            <div className="p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-blue-300">
              Complete your profile to get started with trading.
            </div>
          )}

          {/* Name Field */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-blue-200">Full Name</label>
            {isEditing ? (
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm"
                placeholder="Enter your full name"
              />
            ) : (
              <div className="p-3 bg-blue-500/5 border border-blue-400/10 rounded-xl text-white">
                {profile?.name || formData.name || "Not set"}
              </div>
            )}
          </div>

          {/* Email Field */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-blue-200">Email Address</label>
            {isEditing ? (
              <input
                type="email"
                value={formData.email}
                onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm"
                placeholder="Enter your email address"
              />
            ) : (
              <div className="p-3 bg-blue-500/5 border border-blue-400/10 rounded-xl text-white">
                {profile?.email || user?.email}
              </div>
            )}
          </div>

          {/* Phone Field */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-blue-200">Phone Number</label>
            {isEditing ? (
              <input
                type="tel"
                value={formData.phone}
                onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm"
                placeholder="Enter your phone number (optional)"
              />
            ) : (
              <div className="p-3 bg-blue-500/5 border border-blue-400/10 rounded-xl text-white">
                {profile?.phone || formData.phone || "Not set"}
              </div>
            )}
          </div>

          {/* Location Field */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-blue-200">Location</label>
            {isEditing ? (
              <input
                type="text"
                value={formData.location}
                onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm"
                placeholder="Enter your location (optional)"
              />
            ) : (
              <div className="p-3 bg-blue-500/5 border border-blue-400/10 rounded-xl text-white">
                {profile?.location || formData.location || "Not set"}
              </div>
            )}
          </div>

          {/* Bio Field */}
          <div className="space-y-2">
            <label className="text-sm font-medium text-blue-200">Bio</label>
            {isEditing ? (
              <textarea
                value={formData.bio}
                onChange={(e) => setFormData({ ...formData, bio: e.target.value })}
                rows={4}
                className="w-full p-3 bg-blue-500/10 border border-blue-400/20 rounded-xl text-white placeholder-blue-300 focus:outline-none focus:ring-2 focus:ring-blue-500/50 backdrop-blur-sm resize-none"
                placeholder="Tell us about yourself and your trading experience (optional)"
              />
            ) : (
              <div className="p-3 bg-blue-500/5 border border-blue-400/10 rounded-xl text-white">
                {profile?.bio || formData.bio || "No bio provided"}
              </div>
            )}
          </div>

          {/* Save Button */}
          {isEditing && (
            <div className="flex justify-between pt-4">
              <Button
                onClick={() => {
                  setIsEditing(false)
                  setError(null)
                  // Reset form data if user cancels
                  if (profile) {
                    setFormData({
                      name: profile.name || "",
                      email: profile.email || user?.email || "",
                      phone: profile.phone || "",
                      location: profile.location || "",
                      bio: profile.bio || ""
                    })
                  }
                }}
                variant="ghost"
                className="text-gray-300 hover:text-white hover:bg-gray-600/20"
                disabled={isLoading}
              >
                Cancel
              </Button>
              <Button
                onClick={handleSave}
                className="bg-gradient-to-r from-blue-600/80 to-cyan-600/80 hover:from-blue-500/90 hover:to-cyan-500/90 text-white"
                disabled={isLoading || !formData.name.trim()}
              >
                {isLoading ? (
                  <div className="w-4 h-4 mr-2 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                ) : (
                  <Save className="w-4 h-4 mr-2" />
                )}
                {profile ? 'Save Changes' : 'Create Profile'}
              </Button>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
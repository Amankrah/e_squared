"use client"

import { useState } from "react"
import { Mail, MessageCircle, Clock, Phone } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Header } from "@/components/header"
import { Footer } from "@/components/footer"

export default function Contact() {
  const [formData, setFormData] = useState({
    name: "",
    email: "",
    subject: "",
    message: ""
  })

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // Handle form submission here
    console.log("Form submitted:", formData)
    // Reset form
    setFormData({ name: "", email: "", subject: "", message: "" })
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    setFormData(prev => ({
      ...prev,
      [e.target.name]: e.target.value
    }))
  }

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      {/* Enhanced Glass Morphism Hero Section */}
      <section className="relative py-20 md:py-32 bg-gradient-to-br from-[#0F0C29] via-[#24243e] to-[#302b63] overflow-hidden">
        {/* Background decorations */}
        <div className="absolute inset-0">
          <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-gradient-to-r from-purple-500/10 to-pink-500/10 rounded-full blur-3xl"></div>
          <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-gradient-to-r from-blue-500/10 to-purple-500/10 rounded-full blur-3xl"></div>
          <div className="absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-gradient-to-r from-emerald-500/5 to-teal-500/5 rounded-full blur-3xl"></div>
        </div>

        <div className="relative container px-4">
          <div className="text-center space-y-8 max-w-4xl mx-auto">
            <h1 className="text-5xl font-bold tracking-tight sm:text-6xl md:text-7xl">
              <span className="bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">
                Get in{" "}
              </span>
              <span className="bg-gradient-to-r from-emerald-400 to-teal-400 bg-clip-text text-transparent">
                Touch
              </span>
            </h1>
            <p className="text-xl text-gray-300 max-w-3xl mx-auto leading-relaxed">
              Have questions about E-Squared Trading? We're here to help you succeed.
              Reach out to our team anytime.
            </p>
          </div>
        </div>
      </section>

      {/* Enhanced Glass Morphism Contact Form and Info */}
      <section className="py-20 md:py-32 bg-gradient-to-b from-[#24243e] via-[#302b63] to-[#0F0C29] relative overflow-hidden">
        <div className="container px-4">
          <div className="grid gap-12 lg:grid-cols-2 lg:gap-16">
            {/* Contact Form */}
            <div className="space-y-8">
              <div className="space-y-6">
                <h2 className="text-4xl font-bold tracking-tight bg-gradient-to-r from-white via-purple-100 to-purple-300 bg-clip-text text-transparent">Send us a Message</h2>
                <p className="text-lg text-gray-300 leading-relaxed">
                  Fill out the form below and we'll get back to you within 24 hours.
                </p>
              </div>

              <Card className="border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl shadow-2xl">
                <CardHeader className="space-y-4">
                  <CardTitle className="text-xl font-bold text-white">Contact Form</CardTitle>
                  <CardDescription className="text-gray-300">
                    Tell us how we can help you with your trading journey.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <form onSubmit={handleSubmit} className="space-y-6">
                    <div className="grid gap-4 md:grid-cols-2">
                      <div className="space-y-2">
                        <Label htmlFor="name" className="text-gray-200 font-medium">Full Name</Label>
                        <Input
                          id="name"
                          name="name"
                          type="text"
                          placeholder="Your full name"
                          value={formData.name}
                          onChange={handleChange}
                          className="border-[rgba(147,51,234,0.3)] bg-[rgba(147,51,234,0.05)] text-white placeholder:text-gray-400 focus:border-[rgba(147,51,234,0.5)] backdrop-blur-sm"
                          required
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="email" className="text-gray-200 font-medium">Email Address</Label>
                        <Input
                          id="email"
                          name="email"
                          type="email"
                          placeholder="your.email@example.com"
                          value={formData.email}
                          onChange={handleChange}
                          className="border-[rgba(147,51,234,0.3)] bg-[rgba(147,51,234,0.05)] text-white placeholder:text-gray-400 focus:border-[rgba(147,51,234,0.5)] backdrop-blur-sm"
                          required
                        />
                      </div>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="subject" className="text-gray-200 font-medium">Subject</Label>
                      <Input
                        id="subject"
                        name="subject"
                        type="text"
                        placeholder="What's this about?"
                        value={formData.subject}
                        onChange={handleChange}
                        className="border-[rgba(147,51,234,0.3)] bg-[rgba(147,51,234,0.05)] text-white placeholder:text-gray-400 focus:border-[rgba(147,51,234,0.5)] backdrop-blur-sm"
                        required
                      />
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="message" className="text-gray-200 font-medium">Message</Label>
                      <textarea
                        id="message"
                        name="message"
                        rows={6}
                        placeholder="Tell us more about your inquiry..."
                        value={formData.message}
                        onChange={handleChange}
                        className="flex w-full rounded-md border border-[rgba(147,51,234,0.3)] bg-[rgba(147,51,234,0.05)] text-white placeholder:text-gray-400 px-3 py-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[rgba(147,51,234,0.5)] disabled:cursor-not-allowed disabled:opacity-50 resize-none backdrop-blur-sm"
                        required
                      />
                    </div>

                    <Button type="submit" size="lg" className="w-full bg-gradient-to-r from-purple-600/80 to-pink-600/80 hover:from-purple-500/90 hover:to-pink-500/90 text-white shadow-lg hover:shadow-xl transition-all duration-300 backdrop-blur-sm border border-purple-400/50 hover:translate-y-0.5">
                      Send Message
                    </Button>
                  </form>
                </CardContent>
              </Card>
            </div>

            {/* Enhanced Glass Morphism Contact Information */}
            <div className="space-y-8">
              <div className="space-y-6">
                <h2 className="text-4xl font-bold tracking-tight bg-gradient-to-r from-white via-blue-100 to-blue-300 bg-clip-text text-transparent">Contact Information</h2>
                <p className="text-lg text-gray-300 leading-relaxed">
                  Multiple ways to reach our support team and get the help you need.
                </p>
              </div>

              <div className="space-y-6">
                <Card className="group hover:shadow-2xl transition-all duration-300 border-2 border-[rgba(16,185,129,0.3)] bg-gradient-to-br from-[rgba(16,185,129,0.1)] to-[rgba(16,185,129,0.02)] backdrop-blur-xl">
                  <CardHeader>
                    <div className="flex items-center space-x-4">
                      <div className="h-12 w-12 bg-gradient-to-br from-emerald-500 to-teal-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform duration-300 shadow-lg">
                        <Mail className="h-6 w-6 text-white" />
                      </div>
                      <div>
                        <CardTitle className="text-lg text-white">Email Support</CardTitle>
                        <CardDescription className="text-gray-300">Get help via email</CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-emerald-200 font-medium">support@e-squaredtradings.com</p>
                    <p className="text-sm text-gray-400 mt-1">
                      We respond within 24 hours
                    </p>
                  </CardContent>
                </Card>

                <Card className="group hover:shadow-2xl transition-all duration-300 border-2 border-[rgba(147,51,234,0.3)] bg-gradient-to-br from-[rgba(147,51,234,0.1)] to-[rgba(147,51,234,0.02)] backdrop-blur-xl">
                  <CardHeader>
                    <div className="flex items-center space-x-4">
                      <div className="h-12 w-12 bg-gradient-to-br from-purple-500 to-pink-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform duration-300 shadow-lg">
                        <MessageCircle className="h-6 w-6 text-white" />
                      </div>
                      <div>
                        <CardTitle className="text-lg text-white">Live Chat</CardTitle>
                        <CardDescription className="text-gray-300">Real-time assistance</CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-purple-200 font-medium">Available in-app</p>
                    <p className="text-sm text-gray-400 mt-1">
                      Monday - Friday, 9 AM - 6 PM EST
                    </p>
                  </CardContent>
                </Card>

                <Card className="group hover:shadow-2xl transition-all duration-300 border-2 border-[rgba(59,130,246,0.3)] bg-gradient-to-br from-[rgba(59,130,246,0.1)] to-[rgba(59,130,246,0.02)] backdrop-blur-xl">
                  <CardHeader>
                    <div className="flex items-center space-x-4">
                      <div className="h-12 w-12 bg-gradient-to-br from-blue-500 to-indigo-600 rounded-xl flex items-center justify-center group-hover:scale-110 transition-transform duration-300 shadow-lg">
                        <Clock className="h-6 w-6 text-white" />
                      </div>
                      <div>
                        <CardTitle className="text-lg text-white">24/7 Support</CardTitle>
                        <CardDescription className="text-gray-300">Always here for you</CardDescription>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent>
                    <p className="text-blue-200 font-medium">Round-the-clock assistance</p>
                    <p className="text-sm text-gray-400 mt-1">
                      Emergency support available
                    </p>
                  </CardContent>
                </Card>
              </div>

              {/* Enhanced Glass Morphism Social Links */}
              <div className="space-y-6">
                <h3 className="text-xl font-semibold text-white">Connect With Us</h3>
                <div className="flex flex-wrap gap-3">
                  <Button variant="outline" size="sm" className="border-[rgba(29,161,242,0.3)] bg-gradient-to-r from-[rgba(29,161,242,0.1)] to-[rgba(29,161,242,0.02)] text-blue-200 hover:bg-[rgba(29,161,242,0.2)] backdrop-blur-sm transition-all duration-300" asChild>
                    <a href="https://twitter.com/esquaredtrading" target="_blank" rel="noopener noreferrer">
                      Twitter
                    </a>
                  </Button>
                  <Button variant="outline" size="sm" className="border-[rgba(34,139,230,0.3)] bg-gradient-to-r from-[rgba(34,139,230,0.1)] to-[rgba(34,139,230,0.02)] text-blue-200 hover:bg-[rgba(34,139,230,0.2)] backdrop-blur-sm transition-all duration-300" asChild>
                    <a href="https://t.me/esquaredtrading" target="_blank" rel="noopener noreferrer">
                      Telegram
                    </a>
                  </Button>
                  <Button variant="outline" size="sm" className="border-[rgba(114,137,218,0.3)] bg-gradient-to-r from-[rgba(114,137,218,0.1)] to-[rgba(114,137,218,0.02)] text-indigo-200 hover:bg-[rgba(114,137,218,0.2)] backdrop-blur-sm transition-all duration-300" asChild>
                    <a href="https://discord.gg/esquaredtrading" target="_blank" rel="noopener noreferrer">
                      Discord
                    </a>
                  </Button>
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
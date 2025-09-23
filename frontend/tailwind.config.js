/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: ["class"],
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: {
        "2xl": "1400px",
      },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          DEFAULT: "hsl(var(--primary))",
          foreground: "hsl(var(--primary-foreground))",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "hsl(var(--destructive))",
          foreground: "hsl(var(--destructive-foreground))",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        // E-Squared custom colors - using CSS variables for theme consistency
        'e-primary': 'hsl(221 83% 53%)',    // #1E40AF Rich Indigo
        'e-secondary': 'hsl(262 83% 58%)',  // #9333EA Purple
        'e-success': 'hsl(142 76% 36%)',    // #10B981 Emerald-500
        'e-warning': 'hsl(32 95% 44%)',     // #F59E0B Amber-500
        'e-error': 'hsl(0 84% 60%)',        // #EF4444 Red-500
        'e-bg-light': 'hsl(210 40% 98%)',   // #F9FAFB Gray-50
        'e-bg-dark': 'hsl(215 28% 17%)',    // #1F2937 Gray-800
        'e-text-light': 'hsl(222 84% 5%)',  // #111827 Gray-900
        'e-text-dark': 'hsl(210 20% 84%)',  // #D1D5DB Gray-300
        // Professional Trading Platform Color Palette
        'indigo': {
          50: 'hsl(238 100% 97%)',   // Very light indigo
          100: 'hsl(238 100% 94%)',  // Light indigo
          200: 'hsl(238 100% 88%)',  // Lighter indigo
          300: 'hsl(238 100% 80%)',  // Light indigo
          400: 'hsl(238 100% 70%)',  // Medium light indigo
          500: 'hsl(238 100% 60%)',  // Primary indigo - #4F46E5
          600: 'hsl(238 100% 50%)',  // Darker indigo
          700: 'hsl(238 100% 40%)',  // Dark indigo
          800: 'hsl(238 100% 30%)',  // Darker indigo
          900: 'hsl(238 100% 20%)',  // Very dark indigo
        },
        'purple': {
          50: 'hsl(270 100% 97%)',   // Very light purple
          100: 'hsl(270 100% 94%)',  // Light purple
          200: 'hsl(270 100% 88%)',  // Lighter purple
          300: 'hsl(270 100% 80%)',  // Light purple
          400: 'hsl(270 100% 70%)',  // Medium light purple
          500: 'hsl(270 100% 60%)',  // Primary purple - #8B5CF6
          600: 'hsl(270 100% 50%)',  // Darker purple
          700: 'hsl(270 100% 40%)',  // Dark purple
          800: 'hsl(270 100% 30%)',  // Darker purple
          900: 'hsl(270 100% 20%)',  // Very dark purple
        },
        'emerald': {
          50: 'hsl(158 100% 97%)',   // Very light emerald
          100: 'hsl(158 100% 94%)',  // Light emerald
          200: 'hsl(158 100% 88%)',  // Lighter emerald
          300: 'hsl(158 100% 80%)',  // Light emerald
          400: 'hsl(158 100% 70%)',  // Medium light emerald
          500: 'hsl(158 100% 50%)',  // Primary emerald - #10B981
          600: 'hsl(158 100% 40%)',  // Darker emerald
          700: 'hsl(158 100% 30%)',  // Dark emerald
          800: 'hsl(158 100% 20%)',  // Darker emerald
          900: 'hsl(158 100% 10%)',  // Very dark emerald
        },
        'amber': {
          50: 'hsl(45 100% 97%)',    // Very light amber
          100: 'hsl(45 100% 94%)',   // Light amber
          200: 'hsl(45 100% 88%)',   // Lighter amber
          300: 'hsl(45 100% 80%)',   // Light amber
          400: 'hsl(45 100% 70%)',   // Medium light amber
          500: 'hsl(45 100% 60%)',   // Primary amber - #F59E0B
          600: 'hsl(45 100% 50%)',   // Darker amber
          700: 'hsl(45 100% 40%)',   // Dark amber
          800: 'hsl(45 100% 30%)',   // Darker amber
          900: 'hsl(45 100% 20%)',   // Very dark amber
        },
        'red': {
          50: 'hsl(0 100% 97%)',     // Very light red
          100: 'hsl(0 100% 94%)',    // Light red
          200: 'hsl(0 100% 88%)',    // Lighter red
          300: 'hsl(0 100% 80%)',    // Light red
          400: 'hsl(0 100% 70%)',    // Medium light red
          500: 'hsl(0 100% 60%)',    // Primary red - #EF4444
          600: 'hsl(0 100% 50%)',    // Darker red
          700: 'hsl(0 100% 40%)',    // Dark red
          800: 'hsl(0 100% 30%)',    // Darker red
          900: 'hsl(0 100% 20%)',    // Very dark red
        },
        // Trading-specific colors
        'profit': 'hsl(158 100% 50%)',    // Green for profits
        'loss': 'hsl(0 100% 60%)',        // Red for losses
        'neutral': 'hsl(220 13% 50%)',    // Gray for neutral
        'accent': 'hsl(238 100% 60%)',    // Primary accent
      },
      borderRadius: {
        lg: "var(--radius)",
        md: "calc(var(--radius) - 2px)",
        sm: "calc(var(--radius) - 4px)",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
      },
    },
  },
  plugins: [require("tailwindcss-animate")],
};
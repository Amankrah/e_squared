# E² Trading Platform - Frontend

**PROPRIETARY SOFTWARE - © 2024-2025 E² Trading Platform. All Rights Reserved.**

*Unauthorized copying, modification, or distribution is strictly prohibited.*

---

Modern, responsive frontend for the E² Trading Platform built with Next.js 15, React 19, and TypeScript.

## ⚠️ Disclaimer

This software is for educational purposes only and does not constitute financial advice. Cryptocurrency trading carries substantial risk. Users are solely responsible for their trading decisions.

## Features

### 🎨 User Interface
- **Dashboard**: Comprehensive overview with strategy performance metrics
- **Strategy Management**: Create, configure, and monitor DCA, Grid, and SMA strategies
- **Exchange Connections**: Connect and manage CEX accounts (Binance)
- **Wallet Connections**: Non-custodial DEX wallet management (Ethereum, BNB Chain, Solana)
- **Backtesting**: Visual backtesting interface with performance charts
- **Portfolio Analytics**: Track performance across all strategies
- **Settings**: User preferences, 2FA, session management

### 🔐 Authentication
- Secure JWT-based authentication
- Two-factor authentication (TOTP)
- Session management with device tracking
- Password change functionality
- Protected routes

### 📊 Components
- **Trading Strategies**: DCA, Grid Trading, SMA Crossover management
- **Exchange Integration**: CEX and DEX connection interfaces
- **Charts**: Interactive performance visualizations with Recharts
- **Forms**: Type-safe forms with validation
- **Modals & Dialogs**: Shadcn/ui components

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **React**: Version 19 with Server Components
- **TypeScript**: Full type safety
- **Styling**: Tailwind CSS 4
- **UI Components**: Shadcn/ui
- **Charts**: Recharts
- **State Management**: React Context
- **API Client**: Type-safe fetch wrapper
- **Form Validation**: React Hook Form + Zod

## Getting Started

### Prerequisites
- Node.js 18+
- npm, yarn, or pnpm
- Backend server running on http://localhost:8080

### Installation

1. **Install dependencies**
   ```bash
   npm install
   ```

2. **Configure environment**
   ```bash
   cp .env.example .env.local
   # Edit .env.local with your backend API URL
   ```

3. **Run development server**
   ```bash
   npm run dev
   ```

   Open [http://localhost:3000](http://localhost:3000) in your browser.

### Environment Variables

Create `.env.local`:
```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

## Project Structure

```
src/
├── app/                      # Next.js App Router pages
│   ├── dashboard/            # Dashboard pages
│   │   ├── page.tsx          # Main dashboard
│   │   ├── exchanges/        # Exchange management
│   │   ├── wallets/          # Wallet connections
│   │   ├── strategies/       # Strategy management
│   │   ├── backtesting/      # Backtesting interface
│   │   ├── portfolio/        # Portfolio analytics
│   │   └── settings/         # User settings
│   ├── login/                # Login page
│   ├── signup/               # Registration page
│   └── page.tsx              # Landing page
├── components/               # React components
│   ├── dashboard/            # Dashboard-specific components
│   │   ├── dashboard-layout.tsx
│   │   ├── dashboard-sidebar.tsx
│   │   ├── exchange-connection.tsx
│   │   ├── wallet-connection-*.tsx
│   │   └── strategy components
│   ├── auth/                 # Authentication components
│   └── ui/                   # Shadcn/ui components
├── contexts/                 # React contexts
│   └── auth-context.tsx      # Authentication state
├── lib/                      # Utilities
│   ├── api.ts                # Type-safe API client
│   └── utils.ts              # Helper functions
└── hooks/                    # Custom React hooks
```

## Available Scripts

```bash
# Development
npm run dev          # Start dev server with hot reload

# Production
npm run build        # Build for production
npm start            # Start production server

# Code Quality
npm run lint         # Run ESLint
npm run type-check   # TypeScript type checking

# Testing
npm test             # Run tests
npm run test:e2e     # Run end-to-end tests
```

## Key Pages

### Public Routes
- `/` - Landing page
- `/login` - User login
- `/signup` - User registration
- `/about` - About page
- `/contact` - Contact page

### Protected Routes (require authentication)
- `/dashboard` - Main dashboard
- `/dashboard/strategies/unified` - All strategies management
- `/dashboard/exchanges` - CEX connections
- `/dashboard/wallets` - DEX wallet connections
- `/dashboard/backtesting` - Strategy backtesting
- `/dashboard/portfolio` - Portfolio analytics
- `/dashboard/settings` - User settings

## API Integration

The frontend communicates with the backend via a type-safe API client (`lib/api.ts`):

```typescript
import { apiClient } from '@/lib/api'

// Example: Create DCA strategy
const strategy = await apiClient.createDCAStrategy({
  symbol: 'BTCUSDT',
  base_amount: '100',
  frequency: { Daily: 1 },
  // ... other config
})

// Example: Get wallet connections
const wallets = await apiClient.getWalletConnections()
```

All API responses are fully typed for development safety.

## Component Library

Built with [Shadcn/ui](https://ui.shadcn.com/) components:
- Button, Input, Form, Select
- Dialog, Modal, Alert
- Tabs, Card, Badge
- Table, Dropdown Menu
- Toast notifications
- And more...

## Styling

Uses Tailwind CSS with custom configuration:
- Dark mode support (default)
- Custom color palette (purple/pink gradients)
- Responsive breakpoints
- Component-based styling
- Glass morphism effects

## Development Guidelines

1. **Type Safety**: Use TypeScript strictly, no `any` types
2. **Component Structure**: Keep components small and focused
3. **State Management**: Use React Context for global state
4. **API Calls**: Always use the type-safe API client
5. **Error Handling**: Display user-friendly error messages
6. **Loading States**: Show loading indicators for async operations
7. **Accessibility**: Follow WCAG guidelines

## Performance Optimizations

- Next.js automatic code splitting
- Image optimization with next/image
- Font optimization with next/font
- Server Components for static content
- Client Components for interactive features
- Lazy loading of heavy components
- Memoization of expensive computations

## Security

- HTTP-only cookies for JWT tokens
- CSRF protection
- XSS prevention through React
- Secure API communication
- Input sanitization
- Protected routes with auth guards

## Browser Support

- Chrome (latest)
- Firefox (latest)
- Safari (latest)
- Edge (latest)

## License

**PROPRIETARY LICENSE**

Copyright © 2024-2025 E² Trading Platform. All Rights Reserved.

This software may not be used, copied, modified, or distributed without explicit written permission.

## Contact

- **Support**: support@esquaredtrading.com
- **Business**: contact@esquaredtrading.com

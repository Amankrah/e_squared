# E² Trading Platform

<div align="center">

**🚀 Professional Cryptocurrency Trading Strategy Platform**

*Automated Trading Strategies for CEX & DEX - Built for Traders, By Traders*

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next-black?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

---

### ⚖️ PROPRIETARY SOFTWARE NOTICE

**This software is proprietary and confidential.**

© 2024-2025 E² Trading Platform. All Rights Reserved.

This software and its associated documentation are proprietary to E² Trading Platform. Unauthorized copying, modification, distribution, or use of this software, via any medium, is strictly prohibited without explicit written permission from the copyright holder.

**Restrictions:**
- ❌ No copying, reproduction, or replication
- ❌ No modification or derivative works
- ❌ No distribution, sublicensing, or transfer
- ❌ No reverse engineering or decompilation
- ❌ No commercial use without licensing agreement

For licensing inquiries, contact: **licensing@esquaredtrading.com**

---

</div>

## 🎯 Overview

E² Trading is a comprehensive fullstack cryptocurrency trading platform that combines the power of centralized exchanges (CEX) and decentralized exchanges (DEX) with professional-grade automated trading strategies. Whether you're a beginner looking for simple DCA strategies or an advanced trader implementing complex grid trading and SMA crossover systems, E² provides the tools you need.

### ✨ Key Features

#### 🤖 **Trading Strategies**
- **Dollar Cost Averaging (DCA)** - 10+ preset strategies from conservative to aggressive
  - RSI-based, Volatility-hunter, Dip-buyer, Dynamic multi-factor
  - Weekend warrior, Business hours, Bear market hunter, and more
- **Grid Trading** - Automated buy-low sell-high within price ranges
- **SMA Crossover** - Moving average crossover signals with trend detection

#### 🏦 **Exchange Integration**
- **Centralized Exchanges (CEX)**
  - Binance (Full support: Spot, Futures USDM)
  - Coinbase, Kraken, Bybit, KuCoin, OKX (Coming soon)
- **Decentralized Exchanges (DEX)**
  - Ethereum: Uniswap V3
  - BNB Chain: PancakeSwap V3
  - Solana: Raydium & Jupiter aggregator

#### 🔐 **Security & Wallet Management**
- Non-custodial wallet connections with AES-256 encryption
- Encrypted API key storage for exchanges
- JWT authentication with bcrypt password hashing
- 2FA support with TOTP
- Session management with device tracking

#### 📊 **Advanced Features**
- **Backtesting Engine** - Test strategies against historical data
- **Real-time Market Data** - DXY index, Bitcoin dominance, M2 money supply
- **Technical Indicators** - SMA, EMA, RSI, MACD, Bollinger Bands, Stochastic
- **Risk Management** - Stop loss, take profit, position sizing
- **Portfolio Analytics** - Performance tracking and insights

#### 🎨 **Modern UI/UX**
- Intuitive dashboard with dark mode
- Real-time strategy monitoring
- Interactive charts with Recharts
- Responsive design for all devices

## 🏗️ Architecture

### Backend (Rust)
- **Framework**: Actix Web 4.4 with async/await throughout
- **Database**: SQLite with SeaORM for type-safe queries
- **Authentication**: JWT tokens with HTTP-only cookies, bcrypt hashing
- **Exchange Integration**:
  - Modular connector system for CEX
  - Production-ready DEX connectors (Uniswap, PancakeSwap, Raydium, Jupiter)
- **Strategy Engine**: Extensible trait-based system with backtesting
- **Security**: HMAC signatures, request validation, CORS protection, encrypted credentials

### Frontend (Next.js)
- **Framework**: Next.js 15 with React 19 and TypeScript
- **Styling**: Tailwind CSS 4 with Shadcn/ui components
- **State Management**: React Context for auth and app state
- **Charts**: Recharts for performance visualization
- **API Client**: Type-safe API client with automatic error handling

## 🎯 Trading Strategies

### 1. **Dollar Cost Averaging (DCA)**

#### Conservative Presets:
- **Conservative** - Simple weekly purchases with basic frequency settings
- **Micro DCA** - High-frequency small daily purchases
- **Weekend Warrior** - Weekend-only execution for reduced market hours exposure
- **Business Hours** - Trades only during specified market hours (9 AM - 4 PM UTC, Mon-Fri)

#### Advanced Presets:
- **Aggressive RSI** - RSI-based execution with 3x multiplier when oversold (<25 RSI), 0.2x when overbought (>75 RSI)
- **Volatility Hunter** - Dynamically adjusts purchase amounts based on market volatility:
  - 2x multiplier during high volatility (>25%)
  - 0.7x multiplier during low volatility (<5%)
- **Dip Buyer** - Multi-tier dip buying system with automatic triggers:
  - 5% price drop: 1.5x base amount (max 5 triggers)
  - 10% price drop: 2.5x base amount (max 3 triggers)
  - 20% price drop: 5x base amount (max 2 triggers)
  - 30% price drop: 10x base amount (max 1 trigger)
- **Balanced Dynamic** - Multi-factor strategy combining:
  - RSI indicator (40% weight)
  - Volatility metrics (30% weight)
  - Market sentiment (20% weight)
  - Trend analysis (10% weight)
  - Max multiplier: 3x, Min multiplier: 0.3x
- **Bear Market Hunter** - Increased accumulation during downtrends with configurable thresholds
- **Risk Managed** - Includes strict position limits, max/min purchase amounts, and volatility pause mechanisms

**Use Cases**: Automated accumulation strategies, market condition adaptation, systematic portfolio building

### 2. **Grid Trading Strategy**

Automated buy-low, sell-high execution within defined price ranges.

**Configuration Options:**
- **Grid Types**: Arithmetic (linear spacing) or Geometric (percentage-based spacing)
- **Price Bounds**: User-defined upper and lower price limits
- **Grid Levels**: Customizable number of grid lines (buy/sell levels)
- **Order Sizing**: Equal or custom sizing per grid level
- **Risk Controls**: Maximum open positions, stop-loss levels, position limits

**Grid Mechanics:**
- Places buy orders at lower grid levels
- Places sell orders at higher grid levels
- Automatically captures profit on each completed grid cycle
- Rebalances grid positions based on price movement

**Use Cases**: Range-bound markets, sideways consolidation, automated market making, scalping

### 3. **SMA Crossover Strategy**

Moving average crossover strategy with trend confirmation and dynamic positioning.

**Configuration Options:**
- **Fast SMA Period**: Shorter moving average (e.g., 20, 50)
- **Slow SMA Period**: Longer moving average (e.g., 50, 200)
- **Trend Filter**: Optional long-term SMA for trend confirmation (e.g., 200)
- **Position Sizing**: Configurable base position size with dynamic adjustments
- **Exit Rules**: Crossover reversal, time-based, or profit target

**Signal Types:**
- **Golden Cross**: Fast SMA crosses above Slow SMA (bullish signal)
- **Death Cross**: Fast SMA crosses below Slow SMA (bearish signal)

**Features:**
- Trend strength calculation
- Signal confirmation with volume/momentum
- Dynamic position sizing based on signal strength
- Multiple exit strategies

**Use Cases**: Trend following, swing trading, automated entry/exit, momentum trading

---

**⚠️ Important Notice**: These strategies are for educational and automation purposes. Past performance does not indicate future results. Users should conduct their own research and testing before deployment.

## 🚀 Quick Start

### Prerequisites
- **Rust** 1.70+ with Cargo
- **Node.js** 18+ with npm/yarn
- **Binance Account** with API keys (optional for demo)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/e-squared-trading.git
   cd e-squared-trading
   ```

2. **Backend Setup**
   ```bash
   cd backend
   
   # Create environment file
   cp .env.example .env
   # Edit .env with your configuration
   
   # Install dependencies and run
   cargo build --release
   cargo run
   ```
   
   Backend will start on `http://localhost:8080`

3. **Frontend Setup**  
   ```bash
   cd frontend
   
   # Install dependencies
   npm install
   
   # Start development server
   npm run dev
   ```
   
   Frontend will start on `http://localhost:3000`

### Environment Variables

#### Backend (.env)
```env
DATABASE_URL=sqlite:./e_squared.db
JWT_SECRET=your-super-secure-jwt-secret-key
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
CORS_ORIGIN=http://localhost:3000

# Optional: Binance API credentials
BINANCE_API_KEY=your-binance-api-key
BINANCE_API_SECRET=your-binance-api-secret
```

#### Frontend (.env.local)
```env
NEXT_PUBLIC_API_URL=http://localhost:8080/api/v1
```

## 📊 API Documentation

### Authentication Endpoints
- `POST /api/v1/auth/signup` - Register new user
- `POST /api/v1/auth/login` - User authentication  
- `GET /api/v1/auth/me` - Get current user info
- `POST /api/v1/auth/change-password` - Update password

### DCA Strategy Management
- `POST /api/v1/strategies/dca` - Create new DCA strategy
- `GET /api/v1/strategies/dca` - List user's strategies
- `PUT /api/v1/strategies/dca/{id}` - Update strategy
- `DELETE /api/v1/strategies/dca/{id}` - Delete strategy
- `POST /api/v1/strategies/dca/{id}/execute` - Manual execution

### Exchange Integration
- `POST /api/v1/exchanges/connect` - Connect exchange account
- `GET /api/v1/exchanges/balances` - Get account balances
- `POST /api/v1/exchanges/test-connection` - Verify API credentials

### Strategy Templates
- `GET /api/v1/strategy-templates` - List all templates
- `GET /api/v1/strategy-templates/{id}` - Get specific template
- `POST /api/v1/strategy-templates/recommend` - Get personalized recommendations

## 🛠️ Development

### Project Structure
```
e-squared/
├── backend/                 # Rust backend
│   ├── src/
│   │   ├── handlers/        # API route handlers
│   │   ├── models/          # Database models
│   │   ├── services/        # Business logic
│   │   ├── strategies/      # Trading algorithms
│   │   ├── exchange_connectors/ # Exchange integrations
│   │   └── utils/           # Shared utilities
│   ├── migrations/          # Database migrations
│   └── Cargo.toml          # Rust dependencies
├── frontend/               # Next.js frontend  
│   ├── src/
│   │   ├── app/            # App router pages
│   │   ├── components/     # React components
│   │   ├── lib/            # Utilities and API client
│   │   └── contexts/       # React contexts
│   └── package.json        # Node dependencies
└── docs/                   # Documentation
```

### Database Schema

**Users & Authentication**
- `users` - User accounts and profiles
- `user_sessions` - Active user sessions

**DCA Strategies**  
- `dca_strategies` - Strategy configurations
- `dca_executions` - Execution history and results

**Exchange Integration**
- `exchange_connections` - User exchange credentials
- `wallet_balances` - Account balance snapshots

### Running Tests
```bash
# Backend tests
cd backend
cargo test

# Frontend tests  
cd frontend
npm test

# Integration tests
npm run test:e2e
```

### Building for Production
```bash
# Backend
cd backend
cargo build --release

# Frontend
cd frontend  
npm run build
npm start
```

## 🔒 Security

- **Authentication**: JWT tokens with secure HTTP-only cookies
- **API Security**: Request signing with HMAC-SHA256
- **Data Encryption**: Sensitive data encrypted at rest
- **Rate Limiting**: Configurable request throttling
- **CORS Protection**: Strict origin validation
- **Input Validation**: Server-side validation for all endpoints

## 📈 Performance

- **Backend**: Optimized Rust with connection pooling
- **Database**: Efficient indexing and query optimization  
- **Frontend**: Next.js with automatic code splitting
- **Caching**: Redis support for session and market data
- **Monitoring**: Structured logging with tracing

## 🤝 Contributing

**This is proprietary software.** Contributions are not accepted from external parties. This codebase is closed-source and maintained exclusively by E² Trading Platform.

For employment or partnership opportunities, contact: **careers@esquaredtrading.com**

## 📄 License

**PROPRIETARY LICENSE**

This software is proprietary and confidential. All rights are reserved by E² Trading Platform.

**Copyright © 2024-2025 E² Trading Platform. All Rights Reserved.**

This software may not be used, copied, modified, or distributed without explicit written permission. See the full proprietary license terms in the header of this document.

## ⚠️ Disclaimer

**IMPORTANT - PLEASE READ CAREFULLY**

This software is provided for educational and informational purposes only. E² Trading Platform:

- **Not Financial Advice**: This platform and its strategies do not constitute financial, investment, trading, or any other type of advice
- **No Guarantees**: Past performance of strategies does not guarantee future results
- **Risk Warning**: Cryptocurrency trading carries substantial risk of loss. You may lose your entire investment
- **User Responsibility**: Users are solely responsible for their own trading decisions and outcomes
- **No Liability**: E² Trading Platform assumes no liability for any losses incurred through use of this software
- **Regulatory Compliance**: Users must ensure compliance with their local laws and regulations
- **Due Diligence**: Users should conduct their own research and consult with qualified financial advisors

**By using this software, you acknowledge and accept all risks associated with cryptocurrency trading.**

## 🙏 Acknowledgments

This platform leverages industry-standard technologies:

- **Binance API** - CEX connectivity
- **Uniswap V3** - Ethereum DEX integration
- **PancakeSwap V3** - BNB Chain DEX integration
- **Raydium & Jupiter** - Solana DEX integrations
- **Actix Web** - High-performance Rust web framework
- **Next.js** - React framework for production
- **Shadcn/ui** - UI component library
- **SeaORM** - Type-safe database operations

## 📞 Support

- **Business Inquiries**: contact@esquaredtrading.com
- **Technical Support**: support@esquaredtrading.com
- **Licensing**: licensing@esquaredtrading.com
- **Careers**: careers@esquaredtrading.com

---

<div align="center">

**E² Trading Platform** - Professional Cryptocurrency Trading Automation

**Copyright © 2024-2025 E² Trading Platform. All Rights Reserved.**

*This is proprietary software. Unauthorized use is prohibited.*

</div>

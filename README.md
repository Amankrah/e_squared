# E² Trading Platform

<div align="center">

**🚀 Professional Cryptocurrency Trading Strategy Platform**

*Automated Trading Strategies for CEX & DEX - Built for Traders, By Traders*

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Next.js](https://img.shields.io/badge/Next-black?style=for-the-badge&logo=next.js&logoColor=white)](https://nextjs.org/)
[![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/tailwindcss-%2338B2AC.svg?style=for-the-badge&logo=tailwind-css&logoColor=white)](https://tailwindcss.com/)

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

### 1. **Ultra Conservative DCA** 
*Perfect for beginners and risk-averse investors*
- 📅 **Frequency**: Bi-weekly investments
- 💰 **Risk Level**: Very Low
- 🎯 **Allocation**: $50 - $5,000
- 🛡️ **Protection**: 15% stop loss, 50% take profit

### 2. **Conservative Steady DCA**
*Simple and predictable for long-term holders*
- 📅 **Frequency**: Weekly investments  
- 💰 **Risk Level**: Low
- 🎯 **Allocation**: $100 - $10,000
- 🛡️ **Protection**: 20% stop loss, no take profit

### 3. **Adaptive Zone DCA** ⭐ *Flagship Strategy*
*Smart adaptation to market conditions*
- 📅 **Frequency**: Daily analysis with dynamic execution
- 💰 **Risk Level**: Moderate
- 🎯 **Allocation**: $500 - $50,000
- 🧠 **Intelligence**: Fear & Greed index + volatility adjustments
- 🛡️ **Protection**: 15% stop loss, 100% take profit

### 4. **Aggressive Momentum DCA**
*High-frequency trading for active investors*
- 📅 **Frequency**: Every 4 hours
- 💰 **Risk Level**: High  
- 🎯 **Allocation**: $1,000 - $100,000
- ⚡ **Features**: Large position sizes, quick profit taking

### 5. **Bear Market Accumulator**
*Specialized for downtrends and market crashes*
- 📅 **Frequency**: Every 3 days
- 💰 **Risk Level**: Moderate
- 🎯 **Focus**: Maximum accumulation during fear
- 📊 **Historical**: 85% returns during 2022 bear market

### 6. **Bull Market Rider** 
*Optimized for uptrends with profit protection*
- 📅 **Frequency**: Every 2 days
- 💰 **Risk Level**: Moderate
- 🎯 **Focus**: Momentum capture with greed avoidance
- 📊 **Historical**: 55% annualized returns in bull markets

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

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Workflow
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Binance API** - Exchange connectivity
- **Actix Web** - High-performance Rust web framework
- **Next.js** - React framework for production
- **Shadcn/ui** - Beautiful UI components
- **SeaORM** - Type-safe database operations

## 📞 Support

- **Documentation**: [docs/](./docs/)
- **Issues**: [GitHub Issues](https://github.com/yourusername/e-squared-trading/issues)
- **X**: [Join our community](https://x.com/esquaredtrading)
- **Email**: contact@esquaredtradings.com

---

<div align="center">

**Built with ❤️ for the crypto community**

⭐ Star us on GitHub if you find this project useful!

</div>

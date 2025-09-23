# E-Squared Backend

A high-performance Rust backend built with Actix Web, featuring authentication and user profile management.

## Features

- **Authentication System**
  - User registration (signup)
  - User login with JWT tokens
  - Password change functionality
  - Protected routes with middleware

- **User Profile Management**
  - Comprehensive user profiles with personal information
  - Address management
  - Social media links
  - Skills and interests tracking
  - Privacy controls (public/private profiles)
  - Profile CRUD operations

- **Performance Optimizations**
  - Async/await throughout
  - Connection pooling with SeaORM
  - Optimized release builds
  - CORS support for frontend integration

## Tech Stack

- **Framework**: Actix Web 4.4
- **Database**: SQLite with SeaORM
- **Authentication**: JWT with bcrypt password hashing
- **Validation**: Validator crate
- **Logging**: Tracing with structured logging
- **Configuration**: Environment-based configuration

## API Endpoints

### Authentication (`/api/v1/auth`)
- `POST /signup` - Register a new user
- `POST /login` - Authenticate user and get JWT token
- `GET /me` - Get current user info (protected)
- `POST /change-password` - Change user password (protected)

### User Profile (`/api/v1/profile`)
- `POST /` - Create user profile (protected)
- `GET /` - Get own profile (protected)
- `PUT /` - Update profile (protected)
- `DELETE /` - Delete profile (protected)

### Public Routes (`/api/v1/public`)
- `GET /profile/{id}` - Get public profile by ID

### Health Check
- `GET /health` - Health check endpoint

## Setup

1. **Environment Configuration**
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

2. **Database Setup**
   ```bash
   # SQLite database will be created automatically when you run the server
   # No additional setup needed
   ```

3. **Run the server**
   ```bash
   cargo run
   ```

## Environment Variables

- `DATABASE_URL` - SQLite database file path
- `JWT_SECRET` - Secret key for JWT token signing
- `SERVER_HOST` - Server host (default: 127.0.0.1)
- `SERVER_PORT` - Server port (default: 8080)
- `CORS_ORIGIN` - Allowed CORS origin for frontend

## User Profile Model

The user profile includes (matching frontend structure):
- Personal info (name, email, phone)
- Location (single field for address)
- Bio (personal description)
- Join date (automatically set)
- Avatar URL (for profile picture)
- Verification status (for verified traders)

## Development

```bash
# Check code
cargo check

# Run tests
cargo test

# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Build for production
cargo build --release
```
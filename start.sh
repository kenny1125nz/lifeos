#!/bin/bash

# LifeOS Startup Script
# This script sets up and starts the LifeOS application

set -e

echo "🚀 Starting LifeOS..."

# Check if we're in the right directory
if [ ! -f "README.md" ]; then
    echo "Error: Please run this script from the lifeos root directory"
    exit 1
fi

# Create necessary directories
echo "📁 Creating directories..."
mkdir -p backend/data
mkdir -p frontend/public/icons

# Check if .env file exists, create from example if not
if [ ! -f "backend/.env" ]; then
    echo "📝 Creating .env file from example..."
    cp backend/.env.example backend/.env
    echo "⚠️  Please edit backend/.env to set your JWT_SECRET and other configuration"
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Build the backend
echo "🔨 Building backend..."
cd backend
cargo build --release

echo "✅ Backend built successfully!"

# Create a simple .env file with default values if not exists
if [ ! -f ".env" ]; then
    cat > .env << EOF
DATABASE_PATH=./data/lifeos.db
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRY=3600
REFRESH_EXPIRY=604800
HOST=0.0.0.0
PORT=3000
STATIC_DIR=../frontend/public
RUST_LOG=info
EOF
    echo "⚠️  Created .env with default values. Please change JWT_SECRET for production!"
fi

echo ""
echo "🎉 LifeOS is ready to start!"
echo ""
echo "To start the backend server:"
echo "  cd backend && cargo run"
echo ""
echo "Or run with specific environment:"
echo "  cd backend && JWT_SECRET=your-secret-key cargo run"
echo ""
echo "Once running, open http://localhost:3000 in your browser"
echo ""
echo "For development, you can also run:"
echo "  cd backend && cargo watch -x run"
echo ""

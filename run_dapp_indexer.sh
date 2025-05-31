#!/bin/bash

# DApp Ranking Indexer Runner Script
# This script runs the DApp checkpoint processor for Sui blockchain analytics

set -e

echo "🚀 Starting DApp Ranking Indexer..."

# Check if .env file exists and load environment variables
if [ -f .env ]; then
    echo "📄 Loading environment variables from .env file..."
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "⚠️ No .env file found. Make sure DATABASE_URL is set."
fi

# Verify required environment variables
if [ -z "$DATABASE_URL" ]; then
    echo "❌ DATABASE_URL environment variable is required"
    exit 1
fi

# Build the DApp indexer
echo "🔨 Building DApp indexer..."
cargo build --release --bin dapp_checkpoint_processor

# Configuration
STARTING_CHECKPOINT=${STARTING_CHECKPOINT:-0}
CONCURRENCY=${CONCURRENCY:-25}
ENABLE_METRICS=${ENABLE_METRICS:-true}
METRICS_PORT=${METRICS_PORT:-9184}

echo "📊 Configuration:"
echo "  Starting checkpoint: $STARTING_CHECKPOINT"
echo "  Concurrency: $CONCURRENCY"
echo "  Metrics enabled: $ENABLE_METRICS"
if [ "$ENABLE_METRICS" = "true" ]; then
    echo "  Metrics port: $METRICS_PORT"
fi

# Set database usage
export USE_DATABASE=true

# Run the DApp checkpoint processor
echo "⏳ Starting DApp ranking checkpoint processor..."
echo "📱 Tracking DApp interactions for 24h DAU ranking..."

if [ "$ENABLE_METRICS" = "true" ]; then
    ./target/release/dapp_checkpoint_processor \
        --starting-checkpoint $STARTING_CHECKPOINT \
        --concurrency $CONCURRENCY \
        --enable-metrics \
        --metrics-port $METRICS_PORT
else
    ./target/release/dapp_checkpoint_processor \
        --starting-checkpoint $STARTING_CHECKPOINT \
        --concurrency $CONCURRENCY
fi 
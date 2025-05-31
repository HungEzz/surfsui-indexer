# DApp Ranking Indexer - Sui Blockchain Analytics

## 📋 Overview

The **DApp Ranking Indexer** is a high-performance Rust application that processes Sui blockchain checkpoints to extract and analyze DApp interaction data. It tracks user interactions with various DApps to calculate real-time rankings based on Daily Active Users (DAU).

Key metrics tracked:
- 📊 **24-hour Daily Active Users (DAU)** per DApp
- 🏆 **Real-time DApp Rankings** based on user activity
- 📱 **DApp Interaction Events** from all transactions
- 📈 **Historical ranking trends**

## 🚀 Key Features

- **Real-time Processing**: Processes Sui blockchain checkpoints as they arrive
- **Thread-safe Architecture**: Concurrent checkpoint processing with shared state management
- **Database Persistence**: PostgreSQL storage for rankings and checkpoint progress
- **DApp Mapping**: Human-readable names for popular DApps
- **Configurable Settings**: Environment-based configuration management
- **Background Jobs**: Automatic ranking updates every 10 minutes
- **Memory Efficient**: Automatic cleanup of old interactions to prevent memory growth
- **Comprehensive Logging**: Detailed interaction tracking and ranking reporting

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Sui Network   │────│  Checkpoint      │────│   DApp          │
│   Checkpoints   │    │  Processor       │    │   Extractor     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   DApp Mapping  │────│  Ranking         │────│   Database      │
│   (Names)       │    │  Calculator      │    │  (PostgreSQL)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Key Components

1. **Checkpoint Processor** (`src/bin/dapp_checkpoint_processor.rs`)
   - Main entry point that coordinates all processing
   - Manages worker pools for concurrent checkpoint processing
   - Handles graceful shutdown and progress tracking

2. **DApp Indexer** (`src/dapp_indexer.rs`)
   - Core business logic for extracting DApp interactions
   - Calculates 24-hour DAU metrics for each DApp
   - Manages interaction storage and cleanup

3. **Configuration Management** (`src/config.rs`)
   - Environment variable loading and validation
   - Thread-safe global configuration access
   - Database and network settings

4. **Database Layer** (`src/database.rs`)
   - PostgreSQL integration using Diesel ORM
   - Atomic operations for data consistency
   - Progress tracking and ranking persistence

## 🛠️ Prerequisites

- **Rust** (1.70+ recommended)
- **PostgreSQL** (12+ recommended)
- **Internet Connection** (for Sui network access)

## 📦 Installation

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd suins-indexer
   ```

2. **Install dependencies:**
   ```bash
   cargo build --release
   ```

3. **Set up PostgreSQL database:**
   ```bash
   # Create database
   createdb dapp_ranking
   
   # Tables will be created automatically on first run
   ```

## ⚙️ Configuration

### Environment Variables Setup

Create a `.env` file in the project root with the following variables:

```env
# ==============================================================================
# DATABASE CONFIGURATION
# ==============================================================================

# PostgreSQL connection string
# Format: postgresql://username:password@host:port/database_name
DATABASE_URL=postgresql://postgres:password@localhost:5432/dapp_ranking

# ==============================================================================
# SUI NETWORK CONFIGURATION
# ==============================================================================

# Sui RPC endpoint (optional - has default)
# Default: https://fullnode.mainnet.sui.io:443
RPC_URL=https://fullnode.mainnet.sui.io:443

# Remote storage URL for downloading checkpoints
# If not provided, assumes checkpoints are available locally
REMOTE_STORAGE=https://checkpoints.mainnet.sui.io

# Directory where checkpoints are stored/downloaded
# Default: ./checkpoints
CHECKPOINTS_DIR=./checkpoints

# Path to backfill progress tracking file
# Used to resume processing from the last checkpoint after restarts
BACKFILL_PROGRESS_FILE_PATH=./backfill_progress/backfill_progress

# ==============================================================================
# OPERATIONAL SETTINGS
# ==============================================================================

# Update interval for ranking calculation (in seconds)
# How often the background job updates rankings and saves to database
# Default: 600 (10 minutes)
UPDATE_INTERVAL_SECONDS=600

# Interaction retention period (in hours)
# How long to keep interactions in memory for 24h calculations
# Default: 25 (gives buffer over 24h)
INTERACTION_RETENTION_HOURS=25

# Checkpoint batch size
# How many checkpoints to process before forcing a database update
# Default: 10
CHECKPOINT_BATCH_SIZE=10

# ==============================================================================
# RUNTIME CONTROL
# ==============================================================================

# Enable/disable database functionality
# Set to "true" for production, "false" for testing without database
USE_DATABASE=true

# Starting checkpoint number (optional)
# If not provided, resumes from last processed checkpoint
# STARTING_CHECKPOINT=12345678
```

## 🚀 Usage

### Running the DApp Ranking Indexer

1. **Using the provided script:**
   ```bash
   chmod +x run_dapp_indexer.sh
   ./run_dapp_indexer.sh
   ```

2. **Direct cargo command:**
   ```bash
   cargo run --release --bin dapp_checkpoint_processor
   ```

3. **Using pre-built binary:**
   ```bash
   ./target/release/dapp_checkpoint_processor
   ```

### Expected Output

```
🚀 Starting DApp Ranking Indexer (24h DAU)
📁 Checkpoints dir: ./checkpoints
💾 Database enabled: true
📱 Tracking DApp interactions for ranking
✅ DApp tables created/verified
✅ Loaded last processed checkpoint: 12345678
⏳ Starting DApp ranking checkpoint processing...

------------------------------------
CHECKPOINT: 12345679
Timestamp: 1703123456789
Found 15 DApp interactions
  📱 FanTV AI: 5 interactions
  📱 Aftermath AMM: 4 interactions
  📱 Suilend: 3 interactions
  📱 Pyth: 2 interactions
  📱 Unknown DApp (0xa2f06318): 1 interactions

🏆 Current Top DApps (24h DAU):
  1. FanTV AI - 1,234 DAU
  2. Aftermath AMM - 987 DAU
  3. Suilend - 756 DAU
  4. 6degrees - 543 DAU
  5. Pyth - 432 DAU
------------------------------------
```

## 📊 Database Schema

The indexer creates two main tables:

### DApp Interactions Table
```sql
CREATE TABLE dapp_interactions (
    id SERIAL PRIMARY KEY,
    package_id VARCHAR NOT NULL,
    sender VARCHAR NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    transaction_digest VARCHAR NOT NULL,
    dapp_name VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### DApp Rankings Table
```sql
CREATE TABLE dapp_rankings (
    id SERIAL PRIMARY KEY,
    rank INTEGER NOT NULL,
    package_id VARCHAR NOT NULL,
    dapp_name VARCHAR NOT NULL,
    dau_24h INTEGER NOT NULL,
    last_update TIMESTAMP NOT NULL,
    checkpoint_number BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## 📁 Project Structure

```
src/
├── bin/
│   └── dapp_checkpoint_processor.rs     # Main binary for DApp ranking
├── dapp_indexer.rs                      # Core DApp indexing logic
├── database.rs                          # Database operations
├── models.rs                            # Data models
├── schema.rs                            # Database schema
├── config.rs                            # Configuration management
└── lib.rs                               # Library exports

scripts/
└── run_dapp_indexer.sh                  # Convenience script

docs/
└── DAPP_INDEXER_README.md              # Detailed documentation
```

## 🔧 Development

### Adding New DApps

To add new DApps to the mapping, edit the `initialize_dapp_mapping()` function in `src/dapp_indexer.rs`:

```rust
mapping.insert("0x<package_id>".to_string(), "DApp Name".to_string());
```

### Extending Functionality

1. **New Metrics**: Add new fields to `DAppRanking` struct
2. **Custom Filters**: Modify `extract_dapp_interactions()` function
3. **Additional Tables**: Update `schema.rs` and `database.rs`

## 🐛 Troubleshooting

### Common Issues

1. **Database Connection Failed**
   - Verify PostgreSQL is running
   - Check DATABASE_URL format
   - Ensure database exists

2. **High Memory Usage**
   - Reduce INTERACTION_RETENTION_HOURS
   - Increase CHECKPOINT_BATCH_SIZE
   - Monitor with system tools

3. **Missing Checkpoints**
   - Check CHECKPOINTS_DIR path
   - Verify REMOTE_STORAGE URL
   - Ensure network connectivity

### Performance Tuning

- **Concurrency**: Adjust worker pool size in main function
- **Batch Size**: Increase CHECKPOINT_BATCH_SIZE for better throughput
- **Memory**: Tune INTERACTION_RETENTION_HOURS based on available RAM
- **Database**: Add indexes on frequently queried columns

## 📈 Monitoring

The indexer provides comprehensive logging for monitoring:

- **Checkpoint Progress**: Track processing speed and current position
- **DApp Activity**: Monitor interaction counts per DApp
- **Rankings**: Real-time ranking updates and changes
- **Database Operations**: Success/failure of data persistence
- **Memory Usage**: Automatic cleanup and retention management

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details.

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * DAPP RANKING CHECKPOINT PROCESSOR
 * 
 * This binary is the main entry point for processing Sui blockchain checkpoints
 * to extract and index DApp interaction data for ranking based on Hourly Active Users (HAU).
 * 
 * Key functionalities:
 * - Processes Sui blockchain checkpoints sequentially
 * - Extracts DApp interactions from all events
 * - Calculates 1h HAU metrics for each DApp
 * - Ranks DApps based on their HAU
 * - Stores data in PostgreSQL database
 * - Provides real-time monitoring via logging
 */

use dotenvy::dotenv;
use mysten_service::metrics::start_basic_prometheus_server;
use prometheus::Registry;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use sui_data_ingestion_core::{
    DataIngestionMetrics, FileProgressStore, IndexerExecutor, ReaderOptions, Worker, WorkerPool,
};
use sui_types::full_checkpoint_content::CheckpointData;
use tokio::sync::{oneshot, Mutex};
use tracing::{info, Level, error};
use async_trait::async_trait;
use anyhow::Result;
use suins_indexer::dapp_indexer::{
    DAppIndexer,
};
use suins_indexer::{init_config, get_config};
use suins_indexer::database::DatabaseManager;

/**
 * DAppIndexerWorker is the main worker that processes each checkpoint for DApp ranking
 * It implements the Worker trait to handle checkpoint data processing
 */
struct DAppIndexerWorker {
    // Thread-safe reference to the DApp indexer instance
    indexer: Arc<Mutex<DAppIndexer>>,
    // Database manager for storing processed data
    db_manager: Arc<DatabaseManager>,
}

impl DAppIndexerWorker {
    /// Creates a new DAppIndexerWorker instance
    /// 
    /// # Arguments
    /// * `indexer` - Arc<Mutex<DAppIndexer>> for thread-safe access to the indexer
    /// * `db_manager` - Database manager instance
    fn new(indexer: Arc<Mutex<DAppIndexer>>, db_manager: Arc<DatabaseManager>) -> Self {
        Self {
            indexer,
            db_manager,
        }
    }
}

/**
 * Implementation of the Worker trait for processing checkpoints
 * This is called for each checkpoint that needs to be processed
 */
#[async_trait]
impl Worker for DAppIndexerWorker {
    type Result = ();
    
    /// Process a single checkpoint and extract DApp interactions
    /// 
    /// # Arguments
    /// * `checkpoint` - The checkpoint data containing all transactions
    /// 
    /// # Returns
    /// * `Result<()>` - Success or error result
    async fn process_checkpoint(&self, checkpoint: &CheckpointData) -> Result<()> {
        // Acquire exclusive access to the indexer (thread-safe)
        let mut indexer = self.indexer.lock().await;
        
        // Process the checkpoint and extract DApp interactions
        let dapp_interactions = indexer.process_checkpoint(checkpoint, Some(&self.db_manager)).await;
        
        // Log detailed information if any DApp interactions were found
        if !dapp_interactions.is_empty() {
            info!("------------------------------------");
            info!("CHECKPOINT: {}", checkpoint.checkpoint_summary.sequence_number);
            info!("Timestamp: {}", checkpoint.checkpoint_summary.timestamp_ms);
            
            // Log detailed information about DApp interactions
            info!("Found {} DApp interactions", dapp_interactions.len());
            
            // Group interactions by DApp for better logging
            let mut dapp_counts = std::collections::HashMap::new();
            for interaction in &dapp_interactions {
                let dapp_name = interaction.dapp_name.as_ref()
                    .unwrap_or(&interaction.package_id);
                *dapp_counts.entry(dapp_name.clone()).or_insert(0) += 1;
            }
            
            // Log interactions per DApp
            for (dapp_name, count) in &dapp_counts {
                info!("  📱 {}: {} interactions", dapp_name, count);
            }
            
            // Display current DApp rankings
            let rankings = indexer.get_dapp_rankings();
            if !rankings.is_empty() {
                info!("🏆 Current Top DApps (1h HAU):");
                for (idx, ranking) in rankings.iter().take(10).enumerate() {
                    info!("  {}. {} - {} HAU", 
                        idx + 1, 
                        ranking.dapp_name, 
                        ranking.dau_1h
                    );
                }
            }
            
            info!("------------------------------------");
        }
        
        Ok(())
    }
}

/**
 * Main function - Entry point of the application
 * Sets up logging, configuration, database, and starts checkpoint processing
 */
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging with INFO level and timestamps
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)        
        .with_target(false)  // Don't show module targets
        .with_ansi(true)     // Enable colored output
        .init();
    
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize application configuration from environment variables
    if let Err(err) = init_config() {
        error!("❌ Failed to initialize configuration: {}", err);
        std::process::exit(1);
    }
    
    // Get the validated configuration
    let config = get_config();
    
    // Use default paths since we removed config options
    let checkpoints_dir = env::var("CHECKPOINTS_DIR")
        .unwrap_or("/home/hungez/Documents/surfsui-indexer/checkpoints".to_string());
    
    // Use default remote storage
    let remote_storage = env::var("REMOTE_STORAGE")
        .ok(); // This returns Option<String>
    
    // Use default backfill progress file path
    let backfill_progress_file_path = env::var("BACKFILL_PROGRESS_FILE")
        .unwrap_or("/home/hungez/Documents/surfsui-indexer/backfill_progress/backfill_progress".to_string());
    
    // Get database connection string from configuration
    let database_url = &config.database_url;
    
    // Check if database functionality should be enabled
    let use_database = env::var("USE_DATABASE")
        .unwrap_or("true".to_string())
        .parse::<bool>()
        .unwrap_or(true);

    // Log startup information
    info!("🚀 Starting DApp Ranking Indexer (1h HAU)"); 
    info!("📁 Checkpoints dir: {}", checkpoints_dir);
    info!("💾 Database enabled: {}", use_database);
    info!("📱 Tracking DApp interactions for ranking");

    // Create channels for graceful shutdown
    let (_exit_sender, exit_receiver) = oneshot::channel();
    
    // Set up progress tracking (remembers last processed checkpoint)
    let progress_store = FileProgressStore::new(PathBuf::from(backfill_progress_file_path));

    // Initialize Prometheus metrics server for monitoring
    let registry: Registry = start_basic_prometheus_server();
    let metrics = DataIngestionMetrics::new(&registry);
    
    // Create the main executor with 1 worker thread
    let mut executor = IndexerExecutor::new(progress_store, 1, metrics);

    // Create a new DAppIndexer instance wrapped in Arc<Mutex> for thread safety
    let indexer = Arc::new(Mutex::new(DAppIndexer::new()));
    
    // Setup database manager
    let db_manager = Arc::new(DatabaseManager::new(database_url).await?);
    
    // Initialize database and load existing data if database is enabled
    if use_database {
        info!("✅ Database manager initialized");
        
        // Reset all data to start fresh
        let mut indexer_locked = indexer.lock().await;
        match indexer_locked.reset_database_and_memory(&db_manager).await {
            Ok(()) => {
                info!("✅ Loaded DApp rankings from database");
                
                // Display top 5 DApps
                let rankings = indexer_locked.get_dapp_rankings();
                if !rankings.is_empty() {
                    info!("🏆 Current Top DApps (1h HAU):");
                    for (idx, ranking) in rankings.iter().take(5).enumerate() {
                        info!("  {}. {} - {} HAU", idx + 1, ranking.dapp_name, ranking.dau_1h);
                    }
                } else {
                    info!("ℹ️ No existing DApp rankings found in database");
                }
            }
            Err(err) => {
                error!("❌ Failed to reset database: {}", err);
                return Err(err.into());
            }
        }
        drop(indexer_locked); // Release the lock
        
        info!("🚀 Starting fresh with clean database and memory");
    }

    // Create worker pool with 25 concurrent workers for processing
    let worker_pool = WorkerPool::new(
        DAppIndexerWorker::new(indexer.clone(), db_manager.clone()),
        "dapp_ranking_indexing".to_string(),
        25, // Number of concurrent workers
    );
    
    // Register the worker pool with the executor
    executor.register(worker_pool).await?;
    
    // Start background job to update database rankings every 2 minutes
    if use_database {
        info!("🔄 Starting background database update job (every 2 minutes)");
        suins_indexer::dapp_indexer::start_ranking_update_job(indexer.clone(), db_manager.clone()).await;
    }
    
    info!("⏳ Starting DApp ranking checkpoint processing...");
    
    // Start processing checkpoints
    // This will run indefinitely, processing new checkpoints as they arrive
    executor
        .run(
            PathBuf::from(checkpoints_dir),    // Local checkpoint storage
            remote_storage,                     // Remote checkpoint source
            vec![],                            // Additional checkpoint sources (empty)
            ReaderOptions::default(),          // Default reading options
            exit_receiver,                     // Graceful shutdown receiver
        )
        .await?;
    
    Ok(())
} 
// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * CONFIGURATION MANAGEMENT MODULE
 * 
 * This module handles configuration settings for the DApp Ranking Indexer.
 * It loads settings from environment variables and provides a global 
 * configuration instance for DApp ranking functionality.
 */

use std::env;
use std::time::Duration;
use anyhow::{Result, Context};
use dotenvy::dotenv;
use std::sync::OnceLock;

/**
 * Configuration structure for the DApp Ranking Indexer
 */
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL database connection string
    pub database_url: String,
    
    /// How often to update rankings and save to database (in seconds)
    /// Default: 120 seconds (2 minutes)
    pub update_interval: Duration,
    
    /// Remote storage URL for downloading checkpoints
    pub remote_storage: String,
    
    /// Path to the file tracking backfill progress
    pub backfill_progress_file_path: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenv().ok();
        
        let config = Config {
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            
            update_interval: Duration::from_secs(
                env::var("UPDATE_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "120".to_string()) // Default: 2 minutes
                    .parse::<u64>()
                    .context("UPDATE_INTERVAL_SECONDS must be a valid number")?
            ),
            
            remote_storage: env::var("REMOTE_STORAGE")
                .unwrap_or_else(|_| "https://checkpoints.mainnet.sui.io".to_string()),
            
            backfill_progress_file_path: env::var("BACKFILL_PROGRESS_FILE_PATH")
                .unwrap_or_else(|_| "backfill_progress/backfill_progress".to_string()),
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        if self.update_interval.as_secs() < 60 {
            return Err(anyhow::anyhow!(
                "UPDATE_INTERVAL_SECONDS must be at least 60 seconds"
            ));
        }
        
        if !self.remote_storage.starts_with("http") {
            return Err(anyhow::anyhow!(
                "REMOTE_STORAGE must be a valid HTTP/HTTPS URL"
            ));
        }
        
        Ok(())
    }
    
    /// Print configuration summary
    pub fn print_summary(&self) {
        println!("📋 DApp Ranking Indexer Configuration:");
        println!("  💾 Database: Connected");
        println!("  ⏱️  Update Interval: {}s", self.update_interval.as_secs());
        println!("  ☁️  Remote Storage: {}", self.remote_storage);
        println!("  📄 Progress File: {}", self.backfill_progress_file_path);
    }
}

// Global configuration instance
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Initialize global configuration from environment variables
pub fn init_config() -> Result<()> {
    let config = Config::from_env()?;
    CONFIG.set(config).map_err(|_| {
        anyhow::anyhow!("Configuration has already been initialized")
    })?;
    Ok(())
}

/// Get reference to global configuration
pub fn get_config() -> &'static Config {
    CONFIG.get().expect("Configuration not initialized. Call init_config() first.")
} 
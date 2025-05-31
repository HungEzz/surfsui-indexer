// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/**
 * DAPP RANKING INDEXER MODULE
 * 
 * This module contains the core logic for ranking DApps on Sui blockchain based on Daily Active Users (DAU).
 * It processes checkpoints to extract DApp interactions, calculates DAU metrics,
 * and manages database storage for rankings.
 * 
 * Key components:
 * - DApp interaction extraction from blockchain transactions
 * - Daily Active Users (DAU) calculation
 * - DApp ranking based on 24h DAU
 * - Database interaction for persistence
 */

use sui_types::full_checkpoint_content::{CheckpointData, CheckpointTransaction};
use tracing::{info, error};
use tokio::sync::Mutex;
use std::sync::Arc;
use crate::database::DatabaseManager;
use crate::models::{DAppInteraction, DAppRanking};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, SystemTime};

/**
 * DAppIndexer is the main struct that processes blockchain data for DApp ranking
 * It maintains state about DApp interactions, rankings, and provides methods for processing
 */
#[derive(Clone)]
pub struct DAppIndexer {
    pub dapp_interactions: Vec<DAppInteraction>,  // All processed DApp interactions (24h only)
    pub dapp_rankings: Vec<DAppRanking>,         // Current 24h DApp rankings
    pub dapp_names: HashMap<String, (String, String)>,       // package_id -> (dapp_name, dapp_type) mapping
    pub last_processed_checkpoint: u64,           // Last checkpoint number processed
}

impl DAppIndexer {
    /// Creates a new DAppIndexer instance with default values
    /// All rankings start empty and will be calculated as interactions are processed
    pub fn new() -> Self {
        Self {
            dapp_interactions: Vec::new(),
            dapp_rankings: Vec::new(),
            dapp_names: Self::initialize_dapp_mapping(),
            last_processed_checkpoint: 0,
        }
    }

    /// Initialize the DApp name and type mapping based on the provided list
    fn initialize_dapp_mapping() -> HashMap<String, (String, String)> {
        let mut mapping = HashMap::new();
        
        // Existing DApp mappings with types
        mapping.insert("0xda12d621169da92ed8af5f6b332b7bec64c840bb49bb3d4206d6739cd76bad14".to_string(), ("FanTV AI".to_string(), "AI".to_string()));
        mapping.insert("0x2cdcc3b1306a49fcd5b8ccded57116ad86ab37a93ba9d91fa1ce06a8d22a21e9".to_string(), ("6degrees".to_string(), "Marketing".to_string()));
        mapping.insert("0xa2f06318d797e3a2ba734069165e164870677f705d95d8a18b6d9aabbd588709".to_string(), ("Aftermath AMM".to_string(), "DEX".to_string()));
        mapping.insert("0xada81624f2be6abd31f2433dac2642a03414cdb20d494314a4d3d889281fb5e".to_string(), ("Pebble".to_string(), "GameFi".to_string()));
        mapping.insert("0x04e20ddf36af412a4096f9014f4a565af9e812db9a05cc40254846cf6ed0ad91".to_string(), ("Pyth".to_string(), "Infra".to_string()));
        mapping.insert("0x9c12f3aa14a449a0a23c066589e269086f021a98939f21158cfacb16d19787c3".to_string(), ("Momentum".to_string(), "DEX".to_string()));
        mapping.insert("0x7ea6e27ad7af6f3b8671d59df1aaebd7c03dddab893e52a714227b2f4fe91519".to_string(), ("7K Aggregator".to_string(), "Aggregator".to_string()));
        mapping.insert("0xb908f3c6fea6865d32e2048c520cdfe3b5c5bbcebb658117c41bad70f52b7ccc".to_string(), ("Claynosaurz".to_string(), "NFT".to_string()));
        mapping.insert("0x21f544aff826a48e6bd5364498454d8487c4a90f84995604cd5c947c06b596c3".to_string(), ("Suilend".to_string(), "Lending".to_string()));
        mapping.insert("0x9df4666296ee324a6f11e9f664e35e7fd6b6e8c9e9058ce6ee9ad5c5343c2f87".to_string(), ("Ika".to_string(), "Infra".to_string()));
        
        
        mapping.insert("0x5306f64e312b581766351c07af79c72fcb1cd25147157fdc2f8ad76de9a3fb6a".to_string(), ("Portal".to_string(), "Bridge".to_string()));
        mapping.insert("0x2476333f61ab625ae25205b6726048295fe8b356d26ca841ddf93c69bbd616c8".to_string(), ("Turbos".to_string(), "DEX".to_string()));
        mapping.insert("0x6f5e582ede61fe5395b50c4a449ec11479a54d7ff8e0158247adfda60d98970b".to_string(), ("Cetus AMM".to_string(), "DEX".to_string()));
        mapping.insert("0x3864c7c59a4889fec05d1aae4bc9dba5a0e0940594b424fbed44cb3f6ac4c032".to_string(), ("Cetus AMM".to_string(), "DEX".to_string()));
        mapping.insert("0x51966dc1d9d3e6d85aed55aa87eb9e78e928b4e74b4844a15ef7e3dfb5af3bae".to_string(), ("Cetus Aggregator".to_string(), "Aggregator".to_string()));
        mapping.insert("0x7cdd26c4aa40c990d5ca780e0919b2de796be9bb41fba461d133bfacb0f677bc".to_string(), ("Cetus Aggregator".to_string(), "Aggregator".to_string()));
        mapping.insert("0x2c68443db9e8c813b194010c11040a3ce59f47e4eb97a2ec805371505dad7459".to_string(), ("Wave".to_string(), "Infra".to_string()));
        mapping.insert("0x6d264cc3d4b7b81a7e3e47403b335d1d933ceb03dacc4328214f10bf8937a239".to_string(), ("NAVI Lending".to_string(), "Lending".to_string()));
        mapping.insert("0x8d196820b321bb3c56863b3eb0dd90a49f9eb52e3473373efcebf4388bf04416".to_string(), ("SpringSui".to_string(), "Liquid Staking".to_string()));
        mapping.insert("0x5a6df33a03a69959065b5e87aecac72d0afff893a1923833a77dcfb0d2f42980".to_string(), ("Metastable".to_string(), "CDP".to_string()));
        
        mapping
    }
    
    /// Process a single checkpoint and extract all DApp interactions
    /// This is the main entry point for processing blockchain data
    /// 
    /// # Arguments
    /// * `data` - The checkpoint data containing all transactions
    /// * `db_manager` - Optional database manager for persistence
    /// 
    /// # Returns
    /// * Vec<DAppInteraction> containing all DApp interactions found in this checkpoint
    pub async fn process_checkpoint(
        &mut self, 
        data: &CheckpointData, 
        db_manager: Option<&DatabaseManager>
    ) -> Vec<DAppInteraction> {
        let mut all_interactions = Vec::new();
        let checkpoint_number = data.checkpoint_summary.sequence_number;
        let checkpoint_timestamp = data.checkpoint_summary.timestamp();

        // Process each transaction in the checkpoint
        for (_tx_index, transaction) in data.transactions.iter().enumerate() {
            // Extract DApp interactions from this transaction
            let interactions = self.extract_dapp_interactions(transaction, checkpoint_timestamp);
            
            // Add to our collection
            self.dapp_interactions.extend(interactions.clone());
            all_interactions.extend(interactions);
        }

        // Log only if we found interactions
        if !all_interactions.is_empty() {
            info!("📦 Checkpoint {}: {} DApp interactions found", 
                  checkpoint_number, all_interactions.len());
        }

        // Always prune old interactions and update rankings to ensure 24h window
        self.prune_old_interactions();
        
        // Update rankings every 10 checkpoints or if we have significant interactions
        // This ensures rankings stay fresh and reflect recent 24h data
        if checkpoint_number % 10 == 0 || all_interactions.len() > 5 {
            self.update_dapp_rankings_24h();
            
            // Save to database if available
            if let Some(db_manager) = db_manager {
                if let Err(err) = self.update_data_in_database(db_manager).await {
                    error!("❌ Failed to update database: {}", err);
                }
            }
        }

        // Update last processed checkpoint
        self.last_processed_checkpoint = checkpoint_number;

        all_interactions
    }
    
    /// Extract DApp interactions from a checkpoint transaction
    /// Identifies when users interact with DApps by analyzing transaction events
    /// 
    /// # Arguments
    /// * `transaction` - The checkpoint transaction to analyze
    /// * `checkpoint_timestamp` - When the checkpoint occurred
    /// 
    /// # Returns
    /// * Vec<DAppInteraction> containing all DApp interactions found
    fn extract_dapp_interactions(&self, transaction: &CheckpointTransaction, checkpoint_timestamp: SystemTime) -> Vec<DAppInteraction> {
        let mut interactions = Vec::new();
        let tx_digest = transaction.transaction.digest().to_string();
        
        // Process events to extract DApp interactions and senders
        if let Some(events) = &transaction.events {
            for event in &events.data {
                // Extract package_id from event
                let package_id = event.package_id.to_string();
                
                // Only process events from our tracked DApps
                if let Some((dapp_name, _dapp_type)) = self.dapp_names.get(&package_id) {
                    // Extract sender from event
                    let sender = event.sender.to_string();
                    
                    if sender.is_empty() {
                        continue;
                    }
                    
                    // Create DApp interaction
                    interactions.push(DAppInteraction {
                        package_id,
                        sender,
                        timestamp: checkpoint_timestamp,
                        transaction_digest: tx_digest.clone(),
                        dapp_name: Some(dapp_name.clone()),
                    });
                }
                // Skip all other package_ids that are not in our tracked list
            }
        }

        interactions
    }

    /// Calculate and update 24-hour DApp rankings based on Daily Active Users (DAU)
    fn update_dapp_rankings_24h(&mut self) {
        let now = SystemTime::now();
        let twenty_four_hours_ago = now - Duration::from_secs(24 * 60 * 60);

        // Count unique users per DApp NAME (not package_id) in the last 24 hours
        // This ensures DApps with multiple package IDs are counted as one unified DApp
        let mut dapp_user_counts: HashMap<String, HashSet<String>> = HashMap::new();

        // Process all DApp interactions from the last 24 hours
        for interaction in &self.dapp_interactions {
            if interaction.timestamp >= twenty_four_hours_ago {
                // Only count interactions for DApps that are in our tracked mapping
                if let Some((dapp_name, _dapp_type)) = self.dapp_names.get(&interaction.package_id) {
                    // Count unique users by DApp NAME, not package_id
                    // This fixes the issue where DApps with multiple package IDs 
                    // would have inflated DAU counts
                    dapp_user_counts
                        .entry(dapp_name.clone()) // Use dapp_name as key instead of package_id
                        .or_insert_with(HashSet::new)
                        .insert(interaction.sender.clone());
                }
            }
        }

        // Convert to rankings - group by DApp name
        let mut rankings: Vec<DAppRanking> = dapp_user_counts
            .into_iter()
            .map(|(dapp_name, users)| {
                // Find the first package_id for this dapp_name (for reference)
                let package_id = self.dapp_names
                    .iter()
                    .find(|(_, (name, _))| name == &dapp_name)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                
                // Get dapp_type for this dapp_name
                let dapp_type = self.dapp_names
                    .iter()
                    .find(|(_, (name, _))| name == &dapp_name)
                    .map(|(_, (_, type_name))| type_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());

                DAppRanking {
                    rank: 0, // Will be set after sorting
                    package_id, // Use first package_id as reference
                    dapp_name,
                    dau_24h: users.len() as u32,
                    last_update: now,
                    dapp_type,
                }
            })
            .collect();

        // Sort by DAU (descending) and assign ranks
        rankings.sort_by(|a, b| b.dau_24h.cmp(&a.dau_24h));
        for (index, ranking) in rankings.iter_mut().enumerate() {
            ranking.rank = (index + 1) as u32;
        }

        // Log top 5 DApps if we have rankings
        if !rankings.is_empty() {
            info!("🏆 Top DApps (24h DAU - Fixed Logic):");
            for ranking in rankings.iter().take(5) {
                info!("  {}. {} - {} DAU", ranking.rank, ranking.dapp_name, ranking.dau_24h);
            }
        }

        self.dapp_rankings = rankings;

        // Note: prune_old_interactions is now called in process_checkpoint
        // to ensure it runs every checkpoint, not just when rankings are updated
    }

    /// Remove interactions older than 24 hours and from untracked DApps to prevent memory growth
    fn prune_old_interactions(&mut self) {
        let twenty_four_hours_ago = SystemTime::now() - Duration::from_secs(24 * 60 * 60);
        let initial_count = self.dapp_interactions.len();
        
        self.dapp_interactions.retain(|interaction| {
            // Keep only interactions that are:
            // 1. Within the last 24 hours
            // 2. From tracked DApps
            interaction.timestamp >= twenty_four_hours_ago && 
            self.dapp_names.contains_key(&interaction.package_id)
        });
        
        let removed_count = initial_count - self.dapp_interactions.len();
        if removed_count > 0 {
            info!("🗑️ Pruned {} old interactions, {} remaining", removed_count, self.dapp_interactions.len());
        }
    }

    /// Save current state to database
    pub async fn update_data_in_database(&self, db_manager: &DatabaseManager) -> Result<()> {
        // Clean up Unknown DApps and untracked interactions first
        db_manager.cleanup_unknown_dapps().await?;
        
        // Save current in-memory rankings directly to database
        // This replaces the database calculation since we don't store interactions in DB
        db_manager.save_rankings_from_memory(&self.dapp_rankings).await?;
        info!("💾 Updated DApp rankings in database");

        Ok(())
    }

    /// Load existing data from database
    pub async fn get_data_from_database(&mut self, db_manager: &DatabaseManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Load existing DApp rankings from database
        let ranking_records = db_manager.get_dapp_rankings().await?;
        
        self.dapp_rankings = ranking_records.into_iter().map(|record| {
            DAppRanking {
                rank: record.rank_position as u32,
                package_id: record.package_id,
                dapp_name: record.dapp_name,
                dau_24h: record.dau_24h as u32,
                last_update: SystemTime::now(), // Use current time since we removed last_update from DB
                dapp_type: record.dapp_type,
            }
        }).collect();
             
        info!("Loaded {} DApp rankings from database", self.dapp_rankings.len());
        Ok(())
    }

    /// Get all DApp interactions
    pub fn get_dapp_interactions(&self) -> &Vec<DAppInteraction> {
        &self.dapp_interactions
    }

    /// Get all DApp rankings
    pub fn get_dapp_rankings(&self) -> &Vec<DAppRanking> {
        &self.dapp_rankings
    }

    /// Get top N DApps by ranking
    pub fn get_top_dapps(&self, limit: usize) -> Vec<DAppRanking> {
        self.dapp_rankings
            .iter()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Clear all existing data and reset to only track the 10 specified DApps
    pub fn reset_to_tracked_dapps_only(&mut self) {
        // Clear all existing interactions
        self.dapp_interactions.clear();
        
        // Clear all existing rankings
        self.dapp_rankings.clear();
        
        info!("Reset indexer: cleared all data. Now tracking only {} DApps", 
              self.dapp_names.len());
    }

    /// Reset both database and in-memory data to start fresh
    pub async fn reset_database_and_memory(&mut self, db_manager: &DatabaseManager) -> Result<()> {
        info!("🔄 Starting complete data reset...");
        
        // Reset database
        db_manager.reset_all_data().await?;
        
        // Reset in-memory data
        self.dapp_interactions.clear();
        self.dapp_rankings.clear();
        self.last_processed_checkpoint = 0;
        
        info!("✅ Complete reset finished - database and memory cleared");
        info!("📱 Now tracking {} DApps from scratch", self.dapp_names.len());
        
        Ok(())
    }
}

/// Start a background job to update rankings periodically
pub async fn start_ranking_update_job(indexer: Arc<Mutex<DAppIndexer>>, db_manager: Arc<DatabaseManager>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(120)); // 2 minutes
        
        loop {
            interval.tick().await;
            
            // Update rankings and prune old data
            let mut indexer_guard = indexer.lock().await;
            
            // Always prune old interactions first
            indexer_guard.prune_old_interactions();
            
            // Update rankings based on current 24h data
            indexer_guard.update_dapp_rankings_24h();
            
            // Save to database
            if let Err(err) = indexer_guard.update_data_in_database(&db_manager).await {
                error!("Failed to update rankings in database: {}", err);
            } else {
                info!("✅ Background job: Updated DApp rankings in database");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dapp_indexer_creation() {
        let indexer = DAppIndexer::new();
        assert_eq!(indexer.dapp_interactions.len(), 0);
        assert_eq!(indexer.dapp_rankings.len(), 0);
        assert!(indexer.dapp_names.len() > 0);
    }
} 
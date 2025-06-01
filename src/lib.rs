// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! DApp Ranking Indexer - A high-performance indexer for Sui blockchain DApp analytics
//! 
//! This crate provides indexing capabilities for Sui blockchain events,
//! specifically focusing on DApp ranking based on Hourly Active Users (HAU).

// Core modules
pub mod config;
pub mod database;
pub mod dapp_indexer;
pub mod models;
pub mod schema;

// Re-export commonly used types
pub use config::{init_config, get_config};
pub use database::DatabaseManager;
pub use dapp_indexer::DAppIndexer;
pub use models::{DAppInteraction, DAppRanking, DAppRankingRecord};

// Re-export Sui types for checkpoint processing
pub use sui_types::full_checkpoint_content::{CheckpointData, CheckpointTransaction};

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::schema::dapp_rankings;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/**
 * DAppInteraction represents a user interaction with a DApp
 * Used for calculating Hourly Active Users (HAU) for ranking
 * This is only used in memory, not stored in database
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAppInteraction {
    pub package_id: String,        // DApp package identifier
    pub sender: String,             // User address who interacted
    pub timestamp: SystemTime,      // When the interaction occurred
    pub transaction_digest: String, // Unique transaction identifier
    pub dapp_name: Option<String>,  // Human-readable DApp name (if mapped)
}

// DApp Ranking Models
#[derive(Queryable, Selectable, Debug, Serialize, Deserialize)]
#[diesel(table_name = dapp_rankings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DAppRankingRecord {
    pub rank_position: i32,
    pub package_id: String,
    pub dapp_name: String,
    pub dau_1h: i32,  // 1-hour Hourly Active Users count
    pub dapp_type: String,
}

#[derive(Insertable, AsChangeset, Debug)]
#[diesel(table_name = dapp_rankings)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewDAppRankingRecord {
    pub rank_position: i32,
    pub package_id: String,
    pub dapp_name: String,
    pub dau_1h: i32,  // 1-hour Hourly Active Users count
    pub dapp_type: String,
}

/**
 * DAppRanking represents the 1h ranking of a DApp based on Hourly Active Users
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAppRanking {
    pub rank: u32,                  // Current ranking position
    pub package_id: String,         // DApp package identifier
    pub dapp_name: String,          // Human-readable DApp name
    pub dau_1h: u32,               // 1-hour Hourly Active Users count
    pub last_update: SystemTime,    // Last time ranking was calculated
    pub dapp_type: String,          // DApp category/type
}

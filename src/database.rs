// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use diesel::prelude::*;
use diesel::sql_query;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use crate::models::{DAppRankingRecord, DAppRanking};
use crate::schema::dapp_rankings;
use anyhow::Result;
use tracing::info;

pub struct DatabaseManager {
    pool: Pool<AsyncPgConnection>,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
        let pool = Pool::builder().build(config).await?;
        Ok(Self { pool })
    }

    pub async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, AsyncDieselConnectionManager<AsyncPgConnection>>> {
        Ok(self.pool.get().await?)
    }

    pub async fn get_top_dapps(
        &self,
        limit: i64,
    ) -> Result<Vec<DAppRankingRecord>> {
        let mut conn = self.get_connection().await?;
        
        let rankings = dapp_rankings::table
            .order(dapp_rankings::rank_position.asc())
            .limit(limit)
            .load::<DAppRankingRecord>(&mut conn)
            .await?;

        Ok(rankings)
    }

    pub async fn get_dapp_rankings(&self) -> Result<Vec<DAppRankingRecord>> {
        let mut conn = self.get_connection().await?;
        
        let rankings = dapp_rankings::table
            .order(dapp_rankings::rank_position.asc())
            .load::<DAppRankingRecord>(&mut conn)
            .await?;

        Ok(rankings)
    }

    pub async fn cleanup_unknown_dapps(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Define tracked package IDs
        let tracked_package_ids = vec![
            // Existing DApps
            "0xda12d621169da92ed8af5f6b332b7bec64c840bb49bb3d4206d6739cd76bad14", // FanTV AI
            "0x2cdcc3b1306a49fcd5b8ccded57116ad86ab37a93ba9d91fa1ce06a8d22a21e9", // 6degrees
            "0xa2f06318d797e3a2ba734069165e164870677f705d95d8a18b6d9aabbd588709", // Aftermath AMM
            "0xada81624f2be6abd31f2433dac2642a03414cdb20d494314a4d3d889281fb5e",  // Pebble
            "0x04e20ddf36af412a4096f9014f4a565af9e812db9a05cc40254846cf6ed0ad91", // Pyth
            "0x9c12f3aa14a449a0a23c066589e269086f021a98939f21158cfacb16d19787c3", // Momentum
            "0x7ea6e27ad7af6f3b8671d59df1aaebd7c03dddab893e52a714227b2f4fe91519", // 7K Aggregator
            "0xb908f3c6fea6865d32e2048c520cdfe3b5c5bbcebb658117c41bad70f52b7ccc", // Claynosaurz
            "0x21f544aff826a48e6bd5364498454d8487c4a90f84995604cd5c947c06b596c3", // Suilend
            "0x9df4666296ee324a6f11e9f664e35e7fd6b6e8c9e9058ce6ee9ad5c5343c2f87", // Ika
            // New DApps
            "0x0000000000000000000000000000000000000000000000000000000000000002", // Sui
            "0x0000000000000000000000000000000000000000000000000000000000000001", // Sui
            "0x5306f64e312b581766351c07af79c72fcb1cd25147157fdc2f8ad76de9a3fb6a", // Portal
            "0x2476333f61ab625ae25205b6726048295fe8b356d26ca841ddf93c69bbd616c8", // Turbos
            "0x6f5e582ede61fe5395b50c4a449ec11479a54d7ff8e0158247adfda60d98970b", // Cetus AMM
            "0x3864c7c59a4889fec05d1aae4bc9dba5a0e0940594b424fbed44cb3f6ac4c032", // Cetus AMM
            "0x51966dc1d9d3e6d85aed55aa87eb9e78e928b4e74b4844a15ef7e3dfb5af3bae", // Cetus Aggregator
            "0x7cdd26c4aa40c990d5ca780e0919b2de796be9bb41fba461d133bfacb0f677bc", // Cetus Aggregator
            "0x2c68443db9e8c813b194010c11040a3ce59f47e4eb97a2ec805371505dad7459", // Wave
            "0x6d264cc3d4b7b81a7e3e47403b335d1d933ceb03dacc4328214f10bf8937a239", // NAVI Lending
            "0x8d196820b321bb3c56863b3eb0dd90a49f9eb52e3473373efcebf4388bf04416", // SpringSui
            "0x5a6df33a03a69959065b5e87aecac72d0afff893a1923833a77dcfb0d2f42980", // Metastable
        ];

        // Delete rankings for Unknown DApps or untracked package IDs
        let delete_rankings_query = format!(
            "DELETE FROM dapp_rankings WHERE dapp_name = 'Unknown DApp' OR package_id NOT IN ({})",
            tracked_package_ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<_>>().join(", ")
        );

        sql_query(&delete_rankings_query).execute(&mut conn).await?;

        info!("Cleaned up Unknown DApps and untracked rankings from database");
        Ok(())
    }

    /// Reset all DApp-related data in the database
    /// This clears all rankings to start fresh
    pub async fn reset_all_data(&self) -> Result<()> {
        let mut conn = self.get_connection().await?;

        info!("🗑️ Resetting all DApp data in database...");

        // Delete all DApp rankings
        let delete_rankings_query = "DELETE FROM dapp_rankings";
        let rankings_deleted = sql_query(delete_rankings_query).execute(&mut conn).await?;

        info!("✅ Database reset complete:");
        info!("  - Deleted {} DApp rankings", rankings_deleted);

        Ok(())
    }

    /// Save rankings from memory directly to database
    /// This method takes in-memory rankings and saves them to the database
    pub async fn save_rankings_from_memory(&self, rankings: &[DAppRanking]) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Clear existing rankings first
        let delete_query = "DELETE FROM dapp_rankings";
        sql_query(delete_query).execute(&mut conn).await?;

        // Insert new rankings if we have any
        if !rankings.is_empty() {
            let values: Vec<String> = rankings.iter().map(|ranking| {
                format!(
                    "({}, '{}', '{}', {}, '{}')",
                    ranking.rank,
                    ranking.package_id.replace("'", "''"), // Escape single quotes
                    ranking.dapp_name.replace("'", "''"),  // Escape single quotes
                    ranking.dau_1h,
                    ranking.dapp_type.replace("'", "''")   // Escape single quotes
                )
            }).collect();

            let insert_query = format!(
                "INSERT INTO dapp_rankings (rank_position, package_id, dapp_name, dau_1h, dapp_type) VALUES {}",
                values.join(", ")
            );

            sql_query(&insert_query).execute(&mut conn).await?;
        }

        Ok(())
    }
} 
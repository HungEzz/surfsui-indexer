-- CONSOLIDATED MIGRATION FOR DAPP RANKINGS DATABASE
-- This file contains all migrations combined for easy database setup
-- Run this on a fresh PostgreSQL database

-- ============================================================================
-- DIESEL INITIAL SETUP (Helper functions)
-- ============================================================================

-- Sets up a trigger for the given table to automatically set a column called
-- `updated_at` whenever the row is modified (unless `updated_at` was included
-- in the modified columns)
CREATE OR REPLACE FUNCTION diesel_manage_updated_at(_tbl regclass) RETURNS VOID AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION diesel_set_updated_at() RETURNS trigger AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- MAIN SCHEMA CREATION
-- ============================================================================

-- Create volume and swap related tables
CREATE TABLE IF NOT EXISTS volume_data (
    id SERIAL PRIMARY KEY,
    period VARCHAR(50) NOT NULL,
    sui_usd_volume NUMERIC NOT NULL DEFAULT 0,
    total_usd_tvl NUMERIC NOT NULL DEFAULT 0,
    last_update TIMESTAMP NOT NULL DEFAULT NOW(),
    last_processed_checkpoint BIGINT NOT NULL DEFAULT 0,
    fees_24h NUMERIC NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS swap_events (
    id SERIAL PRIMARY KEY,
    pool_id VARCHAR(255) NOT NULL,
    amount_in NUMERIC NOT NULL,
    amount_out NUMERIC NOT NULL,
    atob BOOLEAN NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    transaction_digest VARCHAR(255) NOT NULL,
    fee_amount NUMERIC NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS liquidity_events (
    id SERIAL PRIMARY KEY,
    pool_id VARCHAR(255) NOT NULL,
    amount_a NUMERIC NOT NULL,
    amount_b NUMERIC NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    transaction_digest VARCHAR(255) NOT NULL
);

-- Create Cetus specific tables
CREATE TABLE IF NOT EXISTS cetus_swap_events (
    id VARCHAR PRIMARY KEY,
    amount_in BIGINT NOT NULL,
    amount_out BIGINT NOT NULL,
    pool VARCHAR NOT NULL,
    pool_id VARCHAR NOT NULL,
    atob BOOLEAN NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS cetus_add_liquidity_events (
    id VARCHAR PRIMARY KEY,
    liquidity VARCHAR NOT NULL,
    after_liquidity VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS cetus_remove_liquidity_events (
    id VARCHAR PRIMARY KEY,
    liquidity VARCHAR NOT NULL,
    after_liquidity VARCHAR NOT NULL
);

-- Create statistics tables
CREATE TABLE IF NOT EXISTS daily_statistics (
    id SERIAL PRIMARY KEY,
    date DATE NOT NULL,
    daily_volume_usd NUMERIC NOT NULL DEFAULT 0,
    daily_tvl_usd NUMERIC NOT NULL DEFAULT 0,
    daily_fees_usd NUMERIC NOT NULL DEFAULT 0,
    swap_count INTEGER NOT NULL DEFAULT 0,
    liquidity_events_count INTEGER NOT NULL DEFAULT 0,
    avg_price_sui_usd NUMERIC,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS hourly_statistics (
    id SERIAL PRIMARY KEY,
    hour_timestamp TIMESTAMP NOT NULL,
    hourly_volume_usd NUMERIC NOT NULL DEFAULT 0,
    hourly_tvl_usd NUMERIC NOT NULL DEFAULT 0,
    hourly_fees_usd NUMERIC NOT NULL DEFAULT 0,
    swap_count INTEGER NOT NULL DEFAULT 0,
    avg_price_sui_usd NUMERIC,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- DAPP RANKINGS TABLE (Final structure with all columns)
-- ============================================================================

CREATE TABLE IF NOT EXISTS dapp_rankings (
    package_id VARCHAR PRIMARY KEY,
    dapp_name VARCHAR NOT NULL,
    dau_1h INTEGER NOT NULL DEFAULT 0,  -- 1-hour Hourly Active Users count
    dapp_type VARCHAR NOT NULL DEFAULT 'Unknown',
    rank_position INTEGER NOT NULL DEFAULT 0,
    last_update TIMESTAMP DEFAULT NOW()
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- Index for dapp_rankings queries
CREATE INDEX IF NOT EXISTS idx_dapp_rankings_rank ON dapp_rankings(rank_position);
CREATE INDEX IF NOT EXISTS idx_dapp_rankings_dau ON dapp_rankings(dau_1h DESC);
CREATE INDEX IF NOT EXISTS idx_dapp_rankings_type ON dapp_rankings(dapp_type);
CREATE INDEX IF NOT EXISTS idx_dapp_rankings_last_update ON dapp_rankings(last_update);

-- Indexes for swap events
CREATE INDEX IF NOT EXISTS idx_swap_events_timestamp ON swap_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_swap_events_pool_id ON swap_events(pool_id);

-- Indexes for liquidity events
CREATE INDEX IF NOT EXISTS idx_liquidity_events_timestamp ON liquidity_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_liquidity_events_pool_id ON liquidity_events(pool_id);

-- Indexes for Cetus events
CREATE INDEX IF NOT EXISTS idx_cetus_swap_timestamp ON cetus_swap_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_cetus_swap_pool_id ON cetus_swap_events(pool_id);

-- ============================================================================
-- SAMPLE DATA (Optional - for testing)
-- ============================================================================

-- Insert sample DApp rankings data
INSERT INTO dapp_rankings (package_id, dapp_name, dau_1h, dapp_type, rank_position, last_update) VALUES
('0xda12d621169da92ed8af5f6b332b7bec64c840bb49bb3d4206d6739cd76bad14', 'FanTV AI', 0, 'AI', 1, NOW()),
('0x2cdcc3b1306a49fcd5b8ccded57116ad86ab37a93ba9d91fa1ce06a8d22a21e9', '6degrees', 0, 'Marketing', 2, NOW()),
('0xa2f06318d797e3a2ba734069165e164870677f705d95d8a18b6d9aabbd588709', 'Aftermath AMM', 0, 'DEX', 3, NOW()),
('0x04e20ddf36af412a4096f9014f4a565af9e812db9a05cc40254846cf6ed0ad91', 'Pyth', 0, 'Infra', 4, NOW()),
('0x9c12f3aa14a449a0a23c066589e269086f021a98939f21158cfacb16d19787c3', 'Momentum', 0, 'DEX', 5, NOW()),
('0x7ea6e27ad7af6f3b8671d59df1aaebd7c03dddab893e52a714227b2f4fe91519', '7K Aggregator', 0, 'Aggregator', 6, NOW()),
('0xb908f3c6fea6865d32e2048c520cdfe3b5c5bbcebb658117c41bad70f52b7ccc', 'Claynosaurz', 0, 'NFT', 7, NOW()),
('0x21f544aff826a48e6bd5364498454d8487c4a90f84995604cd5c947c06b596c3', 'Suilend', 0, 'Lending', 8, NOW()),
('0x9df4666296ee324a6f11e9f664e35e7fd6b6e8c9e9058ce6ee9ad5c5343c2f87', 'Ika', 0, 'Infra', 9, NOW()),
('0x5306f64e312b581766351c07af79c72fcb1cd25147157fdc2f8ad76de9a3fb6a', 'Portal', 0, 'Bridge', 10, NOW()),
('0x2476333f61ab625ae25205b6726048295fe8b356d26ca841ddf93c69bbd616c8', 'Turbos', 0, 'DEX', 11, NOW()),
('0x6f5e582ede61fe5395b50c4a449ec11479a54d7ff8e0158247adfda60d98970b', 'Cetus AMM', 0, 'DEX', 12, NOW()),
('0x3864c7c59a4889fec05d1aae4bc9dba5a0e0940594b424fbed44cb3f6ac4c032', 'Cetus AMM', 0, 'DEX', 13, NOW()),
('0x51966dc1d9d3e6d85aed55aa87eb9e78e928b4e74b4844a15ef7e3dfb5af3bae', 'Cetus Aggregator', 0, 'Aggregator', 14, NOW()),
('0x7cdd26c4aa40c990d5ca780e0919b2de796be9bb41fba461d133bfacb0f677bc', 'Cetus Aggregator', 0, 'Aggregator', 15, NOW()),
('0x2c68443db9e8c813b194010c11040a3ce59f47e4eb97a2ec805371505dad7459', 'Wave', 0, 'Infra', 16, NOW()),
('0x8d196820b321bb3c56863b3eb0dd90a49f9eb52e3473373efcebf4388bf04416', 'SpringSui', 0, 'Liquid Staking', 17, NOW())
ON CONFLICT (package_id) DO NOTHING;

-- ============================================================================
-- COMPLETION MESSAGE
-- ============================================================================

-- Database setup completed successfully!
-- Tables created:
-- - dapp_rankings (main table for DApp rankings with DAU 1h data)
-- - volume_data, swap_events, liquidity_events (trading data)
-- - cetus_* tables (Cetus protocol specific data)
-- - daily_statistics, hourly_statistics (aggregated stats)
-- 
-- Ready for indexer to populate with real-time data! 
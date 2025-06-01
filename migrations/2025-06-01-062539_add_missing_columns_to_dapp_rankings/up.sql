-- Your SQL goes here

-- Add missing columns to dapp_rankings table
ALTER TABLE dapp_rankings ADD COLUMN IF NOT EXISTS rank_position INTEGER DEFAULT 0;
ALTER TABLE dapp_rankings ADD COLUMN IF NOT EXISTS last_update TIMESTAMP DEFAULT NOW();

-- Update existing records to have proper rank_position based on dau_1h
UPDATE dapp_rankings SET rank_position = subquery.row_number
FROM (
    SELECT package_id, ROW_NUMBER() OVER (ORDER BY dau_1h DESC) as row_number
    FROM dapp_rankings
) AS subquery
WHERE dapp_rankings.package_id = subquery.package_id;

-- Update last_update for all existing records
UPDATE dapp_rankings SET last_update = NOW() WHERE last_update IS NULL;

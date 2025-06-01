-- Your SQL goes here

-- Rename dau_24h column to dau_1h to better reflect 1-hour HAU calculation
ALTER TABLE dapp_rankings RENAME COLUMN dau_24h TO dau_1h;

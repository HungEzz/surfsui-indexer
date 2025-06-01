-- This file should undo anything in `up.sql`

-- Revert: Rename dau_1h column back to dau_24h
ALTER TABLE dapp_rankings RENAME COLUMN dau_1h TO dau_24h;

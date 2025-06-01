-- This file should undo anything in `up.sql`

-- Rollback: Remove the added columns
ALTER TABLE dapp_rankings DROP COLUMN IF EXISTS rank_position;
ALTER TABLE dapp_rankings DROP COLUMN IF EXISTS last_update;

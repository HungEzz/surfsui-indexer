-- Test script to check dapp_rankings table schema
\d dapp_rankings;

-- Check if data exists
SELECT COUNT(*) as total_records FROM dapp_rankings;

-- Show sample data if exists
SELECT * FROM dapp_rankings LIMIT 3; 
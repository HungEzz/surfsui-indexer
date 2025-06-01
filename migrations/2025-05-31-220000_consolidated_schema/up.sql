-- Create volume and swap related tables
CREATE TABLE IF NOT EXISTS volume_data (
    id SERIAL PRIMARY KEY,
    package_id VARCHAR NOT NULL,
    volume_amount NUMERIC NOT NULL DEFAULT 0,
    volume_usd NUMERIC NOT NULL DEFAULT 0,
    timestamp TIMESTAMP NOT NULL,
    UNIQUE(package_id, timestamp)
);

CREATE TABLE IF NOT EXISTS swap_events (
    id SERIAL PRIMARY KEY,
    package_id VARCHAR NOT NULL,
    amount_a NUMERIC NOT NULL,
    amount_b NUMERIC NOT NULL,
    timestamp TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS liquidity_events (
    id SERIAL PRIMARY KEY,
    package_id VARCHAR NOT NULL,
    amount_a NUMERIC NOT NULL,
    amount_b NUMERIC NOT NULL,
    timestamp TIMESTAMP NOT NULL,
    event_type VARCHAR NOT NULL
);

-- Create DApp rankings table with final structure
CREATE TABLE IF NOT EXISTS dapp_rankings (
    package_id VARCHAR NOT NULL,
    dapp_name VARCHAR,
    dau_1h INTEGER NOT NULL DEFAULT 0, -- 1-hour Hourly Active Users count
    total_interactions BIGINT NOT NULL DEFAULT 0,
    first_seen_timestamp TIMESTAMP,
    dapp_type VARCHAR NOT NULL DEFAULT 'Unknown',
    PRIMARY KEY (package_id)
); 
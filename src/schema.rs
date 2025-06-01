// @generated automatically by Diesel CLI.

diesel::table! {
    cetus_add_liquidity_events (id) {
        id -> Varchar,
        liquidity -> Varchar,
        after_liquidity -> Varchar,
    }
}

diesel::table! {
    cetus_remove_liquidity_events (id) {
        id -> Varchar,
        liquidity -> Varchar,
        after_liquidity -> Varchar,
    }
}

diesel::table! {
    cetus_swap_events (id) {
        id -> Varchar,
        amount_in -> Int8,
        amount_out -> Int8,
        pool -> Varchar,
        pool_id -> Varchar,
        atob -> Bool,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    daily_statistics (id) {
        id -> Int4,
        date -> Date,
        daily_volume_usd -> Numeric,
        daily_tvl_usd -> Numeric,
        daily_fees_usd -> Numeric,
        swap_count -> Int4,
        liquidity_events_count -> Int4,
        avg_price_sui_usd -> Nullable<Numeric>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    dapp_rankings (package_id) {
        rank_position -> Int4,
        package_id -> Varchar,
        dapp_name -> Varchar,
        dau_1h -> Int4,
        dapp_type -> Varchar,
        last_update -> Nullable<Timestamp>,
    }
}

diesel::table! {
    hourly_statistics (id) {
        id -> Int4,
        hour_timestamp -> Timestamp,
        hourly_volume_usd -> Numeric,
        hourly_tvl_usd -> Numeric,
        hourly_fees_usd -> Numeric,
        swap_count -> Int4,
        avg_price_sui_usd -> Nullable<Numeric>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    liquidity_events (id) {
        id -> Int4,
        #[max_length = 255]
        pool_id -> Varchar,
        amount_a -> Numeric,
        amount_b -> Numeric,
        timestamp -> Timestamp,
        #[max_length = 255]
        transaction_digest -> Varchar,
    }
}

diesel::table! {
    swap_events (id) {
        id -> Int4,
        #[max_length = 255]
        pool_id -> Varchar,
        amount_in -> Numeric,
        amount_out -> Numeric,
        atob -> Bool,
        timestamp -> Timestamp,
        #[max_length = 255]
        transaction_digest -> Varchar,
        fee_amount -> Numeric,
    }
}

diesel::table! {
    volume_data (id) {
        id -> Int4,
        #[max_length = 50]
        period -> Varchar,
        sui_usd_volume -> Numeric,
        total_usd_tvl -> Numeric,
        last_update -> Timestamp,
        last_processed_checkpoint -> Int8,
        fees_24h -> Numeric,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cetus_add_liquidity_events,
    cetus_remove_liquidity_events,
    cetus_swap_events,
    daily_statistics,
    dapp_rankings,
    hourly_statistics,
    liquidity_events,
    swap_events,
    volume_data,
);

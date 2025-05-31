// @generated automatically by Diesel CLI.

diesel::table! {
    dapp_rankings (package_id) {
        rank_position -> Int4,
        package_id -> Varchar,
        dapp_name -> Varchar,
        dau_24h -> Int4,
        dapp_type -> Varchar,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    dapp_rankings,
);

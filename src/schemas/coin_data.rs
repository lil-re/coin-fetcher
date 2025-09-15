use crate::schemas::coins::coins;

diesel::table! {
    coin_data (id) {
        id -> Integer,
        coin_id -> Integer,
        price -> Double,
        market_cap -> Double,
        ts -> Timestamp,
    }
}

diesel::joinable!(coin_data -> coins (coin_id));
diesel::allow_tables_to_appear_in_same_query!(coins, coin_data);

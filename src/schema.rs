// @generated automatically by Diesel CLI.

diesel::table! {
    coin_data (id) {
        id -> Integer,
        coin_id -> Integer,
        price -> Double,
        market_cap -> Double,
        ts -> Timestamp,
    }
}

diesel::table! {
    coins (id) {
        id -> Integer,
        #[max_length = 24]
        uuid -> Varchar,
        #[max_length = 24]
        symbol -> Varchar,
        #[max_length = 128]
        name -> Varchar,
    }
}

diesel::joinable!(coin_data -> coins (coin_id));

diesel::allow_tables_to_appear_in_same_query!(coin_data, coins,);

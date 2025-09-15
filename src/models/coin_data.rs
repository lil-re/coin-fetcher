use crate::schemas::coin_data::{coin_data};
use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Debug, Queryable)]
pub struct CoinData {
    pub id: i32,
    pub coin_id: i32,
    pub price: f64,
    pub market_cap: f64,
    pub ts: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = coin_data)]
pub struct NewCoinData {
    pub coin_id: i32,
    pub price: f64,
    pub market_cap: f64,
}

use crate::schemas::coin_data::{coin_data};
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = coin_data)]
pub struct NewCoinData {
    pub coin_id: i32,
    pub price: f64,
    pub market_cap: f64,
}

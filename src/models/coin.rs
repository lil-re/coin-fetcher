use diesel::prelude::*;
use crate::schemas::coins::{coins};

#[derive(Debug, Queryable)]
pub struct Coin {
    pub id: i32,
    pub uuid: String,
    pub symbol: String,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = coins)]
pub struct NewCoin<'a> {
    pub uuid: &'a str,
    pub symbol: &'a str,
    pub name: &'a str,
}

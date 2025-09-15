use diesel::prelude::*;
use crate::schemas::coins::{coins};

#[derive(Insertable)]
#[diesel(table_name = coins)]
pub struct NewCoin<'a> {
    pub uuid: &'a str,
    pub symbol: &'a str,
    pub name: &'a str,
}

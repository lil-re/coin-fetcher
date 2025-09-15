extern crate diesel;

mod schemas;
mod models;

use reqwest::Client;
use serde::Deserialize;
use governor::{Quota, RateLimiter};
use dotenv::dotenv;
use std::time::Duration;
use std::num::NonZeroU32;
use std::env;
use chrono::{NaiveTime, Utc, TimeZone};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::models::coin_data::NewCoinData;

use crate::schemas::coins::coins::table as coins_table;
use crate::schemas::coins::coins::columns::{id as coin_id, uuid as coin_uuid};
use crate::schemas::coin_data::coin_data::table as coin_data_table;

const DEFAULT_PAGE: usize = 20;
const DEFAULT_LIMIT: usize = 100;
const DEFAULT_RATE: u64 = 2000;
const DEFAULT_HOUR: u32 = 20;
const DEFAULT_MINUTE: u32 = 32;
const DEFAULT_SECOND: u32 = 0;
const DEFAULT_PRICE: f64 = 0.0;

#[derive(Debug, Deserialize, Clone)]
struct ApiCoin {
    uuid: String,
    symbol: String,
    name: String,
    #[serde(rename = "price")]
    price: String,
    #[serde(rename = "marketCap")]
    market_cap: String,
}

#[derive(Debug, Deserialize)]
struct ApiCoinsList {
    coins: Vec<ApiCoin>,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: ApiCoinsList,
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", url))
}

fn insert_or_update_coins(conn: &mut SqliteConnection, coin_list: &[ApiCoin]) {
    use diesel::sql_query;

    // Simple SQL string escape for single quotes
    fn sql_escape(s: &str) -> String {
        s.replace('\'', "''")
    }

    if coin_list.is_empty() {
        return;
    }

    let mut values = String::new();
    for c in coin_list {
        let uuid = sql_escape(&c.uuid);
        let symbol = sql_escape(&c.symbol);
        let name = sql_escape(&c.name);
        values.push_str(&format!("('{}','{}','{}'),", uuid, symbol, name));
    }
    values.pop(); // remove last comma

    let query = format!(
        "INSERT INTO coins (uuid, symbol, name)
         VALUES {}
         ON CONFLICT(uuid) DO UPDATE SET
            symbol = excluded.symbol,
            name = excluded.name;",
        values
    );

    sql_query(query).execute(conn).expect("Insert coins failed");
}

fn insert_coin_data(conn: &mut SqliteConnection, coin_list: &[ApiCoin]) {
    for c in coin_list {
        let current_coin_id = coins_table.
            filter(coin_uuid.eq(&c.uuid))
            .select(coin_id)
            .first::<i32>(conn);

        if let Ok(found_id) = current_coin_id {
            let price = c.price.parse::<f64>().unwrap_or(DEFAULT_PRICE);
            let market_cap = c.market_cap.parse::<f64>().unwrap_or(DEFAULT_PRICE);
            let new_data = NewCoinData {
                coin_id: found_id,
                price,
                market_cap,
            };

            diesel::insert_into(coin_data_table)
                .values(&new_data)
                .execute(conn)
                .expect("Insert coin_data failed");
        }
    }
}

async fn fetch_coins() -> Vec<ApiCoin> {
    let api_key = env::var("API_KEY").expect("Please set API_KEY");
    let api_url = env::var("API_URL").expect("Please set API_URL");
    let pages = env::var("PAGES").expect("Please set PAGES").parse::<usize>().unwrap_or_else(|_| DEFAULT_PAGE);

    let client = Client::new();
    let limiter = RateLimiter::direct(
        Quota::with_period(Duration::from_millis(DEFAULT_RATE)).unwrap()
            .allow_burst(NonZeroU32::new(1).unwrap())
    );

    let mut coin_list: Vec<ApiCoin> = Vec::new();

    for i in 0..pages {
        limiter.until_ready().await;
        let current_limit = i * DEFAULT_LIMIT;
        let url = format!("{}{}", api_url, current_limit);

        let resp = client.get(&url)
            .header("x-access-token", &api_key)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let text = r.text().await.unwrap();
                let parsed: Result<ApiResponse, _> = serde_json::from_str(&text);
                match parsed {
                    Ok(data) => coin_list.extend(data.data.coins),
                    Err(e) => eprintln!("JSON parse error on page {}: {}", i, e),
                }
            }
            Ok(r) => eprintln!("Page {} - API request failed with status: {}", i, r.status()),
            Err(e) => eprintln!("Page {} - Request error: {}", i, e),
        }
    }

    coin_list
}

// Compute the next delay until the desired HH:MM (UTC). Default: 02:17 UTC if env missing/invalid.
fn next_delay_until_target_utc() -> Duration {
    let target_time = NaiveTime::from_hms_opt(DEFAULT_HOUR, DEFAULT_MINUTE, DEFAULT_SECOND).expect("valid time");
    let now_utc = Utc::now();
    let today_naive = now_utc.date_naive();
    let mut target_dt = Utc.from_utc_datetime(&today_naive.and_time(target_time));

    if target_dt <= now_utc {
        target_dt = target_dt + chrono::Duration::days(1);
    }

    let diff = target_dt - now_utc;
    diff.to_std().unwrap_or(Duration::from_secs(0))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut conn = establish_connection();

    loop {
        let delay = next_delay_until_target_utc();
        let next_at = Utc::now() + chrono::Duration::from_std(delay).unwrap_or_default();

        println!(
            "[scheduler] Next run at {} UTC (in {:?})",
            next_at.format("%Y-%m-%d %H:%M:%S"),
            delay
        );

        tokio::time::sleep(delay).await;

        println!("[job] Starting coin fetch...");

        let coin_list = fetch_coins().await;
        insert_or_update_coins(&mut conn, &coin_list);
        insert_coin_data(&mut conn, &coin_list);

        println!("[job] Completed: inserted/updated {} coins", coin_list.len());
    }
}

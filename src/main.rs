use chrono::{self, Days};
use db::Database;

mod config;
mod db;
mod parse_xml;

#[tokio::main]
async fn main() {
    let config = config::Config::from_file(".env");
    let database = init_db();
    println!("config = {:?}", config);
    let url = build_url(&config);
    println!("url = {}", url);
    let body = reqwest::get(url).await.unwrap().text().await.unwrap();
    let prices = parse_xml::parse_price_xml(&body);
    for price in prices {
        match database.insert_price(&price) {
            Err(err) => println!("Error inserting {} to database: {}", price, err),
            _ => {}
        }
    }
}

fn build_url(config: &config::Config) -> String {
    let now = chrono::Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = now.checked_add_days(Days::new(1)).unwrap().format("%Y%m%d2300").to_string();
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart={}&periodEnd={}", config.api_key, config.area, config.area, start, end)
}

fn init_db() -> Database {
    let database = db::Database::get_connection();
    match database.create_schema() {
        Ok(_) => println!("Database schema created"),
        Err(err) => println!("Error creating database schema: {}", err),
    }
    database 
}

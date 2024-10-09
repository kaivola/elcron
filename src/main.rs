mod config;

use chrono::{self, Days};

#[tokio::main]
async fn main() {
    let config = config::Config::from_file(".env");
    println!("config = {:?}", config);
    let url = build_url(&config);
    println!("url = {}", url);
    let _body = reqwest::get(url).await.unwrap().text().await.unwrap();
}

fn build_url(config: &config::Config) -> String {
    let now = chrono::Utc::now();
    let start = now.format("%Y%m%d0000").to_string();
    let end = now.checked_add_days(Days::new(1)).unwrap().format("%Y%m%d2300").to_string();
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart={}&periodEnd={}", config.api_key, config.area, config.area, start, end)
}

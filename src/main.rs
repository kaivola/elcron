use chrono::{Days, Duration, Local, NaiveTime};
use env_logger::Env;
use log::{error, info};
use std::thread;

use config::Config;
use db::Database;

mod config;
mod db;
mod xml_parser;

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace");
    env_logger::init_from_env(env);
    let db = db::Database::get_connection();
    init_db(&db);
    let config = config::Config::from_file(".env");
    let handle = thread::spawn(|| update_price_data(config, db));
    handle.join().unwrap();
}

#[tokio::main]
async fn update_price_data(config: Config, db: Database) {
    loop {
        info!("Fetching price data from ENTSO-E API");
        let url = build_url(&config);
        info!("URL: {}", url.replace(&config.api_key, "***"));
        let response = reqwest::get(url).await.unwrap();
        info!("Response - status: {}, size: {} bytes", response.status(), response.content_length().unwrap());
        let body = response.text().await.unwrap();
        let prices = xml_parser::parse_price_xml(&body);
        let mut count = 0;
        for price in prices {
            match db.insert_price(&price) {
                Err(err) => error!("Error inserting {} to database: {}", price, err),
                _ => count += 1,
            }
        }
        info!("Inserted {} Price entities to database", count);
        let now = Local::now();
        let time = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
        let target = now.with_time(time).unwrap() + Duration::days(1);
        let duration = target - now;
        info!("Sleeping until: {}", target);
        thread::sleep(duration.to_std().unwrap());
    }
}

fn build_url(config: &config::Config) -> String {
    let tomorrow = chrono::Utc::now().checked_add_days(Days::new(1)).unwrap();
    info!("Fetching data for: {}", tomorrow.format("%Y-%m-%d"));
    let start = tomorrow.format("%Y%m%d0000").to_string();
    let end = tomorrow.format("%Y%m%d2300")
        .to_string();
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart={}&periodEnd={}", config.api_key, config.area, config.area, start, end)
}

fn init_db(db: &Database) {
    info!("Initializing database schema");
    match db.create_schema() {
        Ok(rows_affected) => if rows_affected == 0 {
            info!("Database schema already exists");
        } else {
            info!("Created database schema");
        },
        Err(err) => error!("Error creating database schema: {}", err),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_build_url() {
        let config = config::Config {
            api_key: "test_api_key".to_string(),
            area: "test_area".to_string(),
        };
        let url = build_url(&config);
        assert!(url.contains("test_api_key"));
        assert!(url.contains("test_area"));
        let tomorrow = chrono::Utc::now().checked_add_days(Days::new(1)).unwrap();
        assert!(url.contains(format!("periodStart={}", tomorrow.format("%Y%m%d0000")).as_str()));
        assert!(url.contains(format!("periodEnd={}", tomorrow.format("%Y%m%d2300")).as_str()));
    }
}
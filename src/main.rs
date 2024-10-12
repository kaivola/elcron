use chrono::{Duration, Local, NaiveTime, Timelike};
use elcron_parser::parse_elcron_file;
use env_logger::Env;
use log::info;
use std::collections::VecDeque;
use std::thread;
use xml_parser::Price;

use config::Config;

mod config;
mod elcron_parser;
mod xml_parser;

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace");
    env_logger::init_from_env(env);
    let config = config::Config::from_file(".env");
    let mut price_queue: VecDeque<Price> = VecDeque::new();
    loop {
        let jobs = parse_elcron_file(".elcron");
        let now = Local::now();
        if now.hour() == 15 || price_queue.is_empty() {
            info!("Updating price data");
            let prices: Vec<Price> = get_price_data(&config).await;
            add_prices_to_queue(prices, &mut price_queue);
        }

        let current_price = match price_queue.pop_front() {
            Some(price) => {
                if price.date != now.format("%Y-%m-%d").to_string() || price.hour != u8::try_from(now.hour()).unwrap() {
                    panic!("Price data is not up to date");
                }
                price
            },
            None => {
                panic!("No price data available");
            }
        };
        info!("The electricity price at {} is {:.2} c/kWh", now.format("%Y-%m-%d %H:%M"), current_price.price);
        for job in jobs {
            if job.should_execute(current_price.price) {
                job.execute();
            }
        }
        
        let next_hour = NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap() + Duration::hours(1);
        let target_time = now.with_time(next_hour).unwrap();
        let duration = target_time - now;
        info!("Sleeping until: {}", target_time);
        thread::sleep(duration.to_std().unwrap());
    }
}

fn add_prices_to_queue(prices: Vec<Price>, price_queue: &mut VecDeque<Price>) {
    let new_prices: Vec<Price> = match price_queue.len() {
        0 => {
            let now = Local::now();
            let date = now.format("%Y-%m-%d").to_string();
            let current_hour = u8::try_from(now.hour()).unwrap();
            prices
                .into_iter()
                .skip_while(|p| p.date == date && p.hour < current_hour)
                .collect()
        }
        _ => {
            let last_price = price_queue.back().unwrap();
            prices
                .into_iter()
                .skip_while(|p| p.date <= last_price.date)
                .skip_while(|p| p.hour < last_price.hour)
                .collect()
        }
    };
    info!("Adding {} new price items to queue", new_prices.len());
    price_queue.extend(new_prices);
}

async fn get_price_data(config: &Config) -> Vec<Price> {
    info!("Fetching price data from ENTSO-E API");
    let url = build_url(config);
    info!("URL: {}", url.replace(&config.api_key, "***"));
    let response = reqwest::get(url).await.unwrap();
    info!(
        "Response - status: {}, size: {} bytes",
        response.status(),
        response.content_length().unwrap(),
    );
    let body = response.text().await.unwrap();
    let mut prices = xml_parser::parse_price_xml(&body);
    prices.sort_by(|a, b| a.date.cmp(&b.date).then(a.hour.cmp(&b.hour)));
    prices
}

fn build_url(config: &Config) -> String {
    let now = Local::now();
    info!("Fetching data starting from: {}", now.format("%Y-%m-%d"));
    let start = now.format("%Y%m%d0000").to_string();
    let tomorrow = now + Duration::days(1);
    let end = tomorrow.format("%Y%m%d2300").to_string();
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart={}&periodEnd={}", config.api_key, config.area, config.area, start, end)
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
        let now = Local::now();
        let tomorrow = now + Duration::days(1);
        assert!(url.contains(format!("periodStart={}", now.format("%Y%m%d0000")).as_str()));
        assert!(url.contains(format!("periodEnd={}", tomorrow.format("%Y%m%d2300")).as_str()));
    }
}

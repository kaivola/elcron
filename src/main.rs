use chrono::{Duration, Local, NaiveTime, Timelike};
use env_logger::Env;
use log::info;
use std::collections::VecDeque;
use std::thread;

use crate::{config::Config, xml_parser::Price, elcron_parser::parse_elcron_file, api::get_price_data};

mod api;
mod config;
mod elcron_parser;
mod xml_parser;

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace");
    env_logger::init_from_env(env);
    let config = Config::from_file(".env");
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
        
        sleep_until_next_hour();
    }
}

fn sleep_until_next_hour() {
    let now = Local::now();
    let next_hour = NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap() + Duration::hours(1);
    let target_time = now.with_time(next_hour).unwrap();
    let duration = target_time - now;
    info!("Sleeping until: {}", target_time);
    thread::sleep(duration.to_std().unwrap());
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

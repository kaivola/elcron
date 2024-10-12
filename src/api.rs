use chrono::{Duration, Local};
use log::info;

use crate::{config::Config, xml_parser::{parse_price_xml, Price}};

pub async fn get_price_data(config: &Config) -> Vec<Price> {
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
    let mut prices = parse_price_xml(&body);
    prices.sort_by(|a, b| a.date.cmp(&b.date).then(a.hour.cmp(&b.hour)));
    prices
}

fn build_url(config: &Config) -> String {
    let now = Local::now();
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
        let config = Config {
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
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
struct Config {
    api_key: String,
    area: String,
}

impl Config {
    fn from_dotenv() -> Self {
        let env_file = read_env_file();
        Self {
            api_key: env_file.get("API_KEY").unwrap_or_else(|| {panic!("API_KEY not found in .env file")}).to_string(),
            area: env_file.get("AREA").unwrap_or_else(|| {panic!("AREA not found in .env file")}).to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_dotenv();
    println!("config = {:?}", config);
    let url = build_url(&config);
    println!("url = {}", url);
    let _body = reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
}

fn build_url(config: &Config) -> String {
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart=202312230000&periodEnd=202312242300", config.api_key, config.area, config.area)
}


fn read_env_file() -> HashMap<String, String> {
    let file = File::open(".env").expect(".env file not found");
    let reader = BufReader::new(file);

    let mut config: HashMap<String, String> = HashMap::new();

    for line in reader.lines() {
        let line = line.unwrap();
        if !line.starts_with('#') {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().trim_matches('"');
                let val = parts[1].trim().trim_matches('"');
                config.insert(String::from(key), String::from(val));
            }
        }
    }
    return config;
}
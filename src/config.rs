use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use std::fs::File;

#[derive(Debug)]
pub struct Config {
    pub api_key: String,
    pub area: String,
}

impl Config {
    pub fn from_file(filename: &str) -> Self {
        let env_file = read_env_file(filename);
        Self {
            api_key: env_file
                .get("API_KEY")
                .unwrap_or_else(|| panic!("API_KEY not found in .env file"))
                .to_string(),
            area: env_file
                .get("AREA")
                .unwrap_or_else(|| panic!("AREA not found in .env file"))
                .to_string(),
        }
    }
}

/// Reads a file containing key-value pairs separated by '=' and returns HashMap
fn read_env_file(filename: &str) -> HashMap<String, String> {
    let file = File::open(filename).expect(".env file not found");
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
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_env_file() {
        let config = read_env_file(".env.test");
        assert!(config.contains_key("API_KEY"));
        assert_eq!(config.get("API_KEY").unwrap(), "test_api_key");
        assert!(config.contains_key("AREA"));
        assert_eq!(config.get("AREA").unwrap(), "test_area");
        assert!(!config.contains_key("COMMENTED_KEY"));
    }

    #[test]
    fn test_config_from_file() {
        let config = Config::from_file(".env.test");
        assert_eq!(config.api_key, "test_api_key");
        assert_eq!(config.area, "test_area");
    }
}
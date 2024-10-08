mod config;


#[tokio::main]
async fn main() {
    let config = config::Config::from_file(".env");
    println!("config = {:?}", config);
    let url = build_url(&config);
    println!("url = {}", url);
    let _body = reqwest::get(url).await.unwrap().text().await.unwrap();
}

fn build_url(config: &config::Config) -> String {
    format!("https://web-api.tp.entsoe.eu/api?securityToken={}&documentType=A44&in_Domain={}&out_Domain={}&periodStart=202312230000&periodEnd=202312242300", config.api_key, config.area, config.area)
}

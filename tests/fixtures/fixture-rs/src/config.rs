/// 默认配置。
#[derive(Debug)]
pub struct Config {
    pub endpoint: String,
    pub timeout_ms: u64,
}

pub fn load_default() -> Config {
    Config {
        endpoint: "http://localhost:8080".to_string(),
        timeout_ms: 5000,
    }
}

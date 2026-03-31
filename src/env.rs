use std::env;

pub fn args() -> Vec<String> {
    env::args().collect()
}

pub fn required(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| {
        format!("{} must be set in the .env file: {:#?}", key, e)
    })
}

pub fn bool(key: &str) -> bool {
    env::var(key).map(|v| v == "true").unwrap_or(false)
}

pub fn with_default(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn u64_with_default(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

pub fn opt_bool(key: &str) -> Option<bool> {
    std::env::var(key).ok().map(|v| v == "true")
}

pub fn opt<T: std::str::FromStr>(key: &str) -> Option<T> {
    std::env::var(key).ok().and_then(|v| v.parse().ok())
}

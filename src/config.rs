use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FaitConfig {
    pub host: String,
    pub port: u16,
    pub backends: Vec<String>,
}

impl Default for FaitConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3750,
            backends: vec![
                "https://auth.spacestation14.com".to_string(),
                "https://auth.playss14.com".to_string(),
            ],
        }
    }
}
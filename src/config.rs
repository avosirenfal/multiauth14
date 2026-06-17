use std::collections::HashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackendDefinition {
    pub url: String,
    pub forbidden_uuids: HashSet<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MulitAuthConfig {
    pub host: String,
    pub port: u16,
    pub backends: Vec<BackendDefinition>,
}

impl Default for MulitAuthConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3750,
            backends: vec![
                BackendDefinition {
                    url: "https://auth.spacestation14.com".to_string(),
                    forbidden_uuids: HashSet::new(),
                },
                BackendDefinition {
                    url: "https://auth.playss14.com".to_string(),
                    forbidden_uuids: HashSet::new(),
                },
            ],
        }
    }
}
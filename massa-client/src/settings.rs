// Copyright (c) 2022 MASSA LABS <info@massa.net>

//! Build here the default client settings from the configuration file toml
use massa_models::config::build_massa_settings;
use massa_time::MassaTime;
use serde::Deserialize;
use std::{net::IpAddr, path::PathBuf};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Settings = build_massa_settings("massa-client", "MASSA_CLIENT");
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub default_node: DefaultNode,
    pub history: usize,
    pub history_file_path: PathBuf,
    pub timeout: MassaTime,
    pub http: HttpSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultNode {
    pub ip: IpAddr,
    pub private_port: u16,
    pub public_port: u16,
    pub api_port: u16,
}

/// Http Client settings.
/// the Http Client settings
#[derive(Debug, Deserialize, Clone)]
pub struct HttpSettings {
    pub max_request_body_size: u32,
    pub request_timeout: MassaTime,
    pub max_concurrent_requests: usize,
    pub certificate_store: String,
    pub id_kind: String,
    pub max_log_length: u32,
    pub headers: Vec<(String, String)>,
}

#[cfg(test)]
#[test]
fn test_load_client_config() {
    let _ = *SETTINGS;
}

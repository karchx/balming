mod defaults;

use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub wifi_ssid: String,
    pub wifi_pass: String,
    pub bot_token: String,
    pub allow_chat_id: i64,
}

impl Default for AppConfig {
    fn default() -> Self {
        let parse_allow_chat_id = defaults::ALLOW_CHAT_ID.parse().unwrap_or(00000);
        Self {
            wifi_ssid: defaults::WIFI_SSID.into(),
            wifi_pass: defaults::WIFI_PASSWORD.into(),
            bot_token: defaults::BOT_TOKEN.into(),
            allow_chat_id: parse_allow_chat_id,
        }
    }
}

#[derive(Deserialize, Default)]
struct ConfigFile {
    ssid: Option<String>,
    pass: Option<String>,
    token: Option<String>,
    chat: Option<i64>,
}

impl AppConfig {
    pub fn load(base: &str) -> Result<Self> {
        let mut cfg = Self::default();
        let path = Path::new(base).join("config.json");
        if !path.exists() {
            return Ok(cfg);
        }

        let data = fs::read_to_string(&path)?;
        let d: ConfigFile = serde_json::from_str(&data)?;

        if let Some(v) = d.ssid {
            cfg.wifi_ssid = v;
        }
        if let Some(v) = d.pass {
            cfg.wifi_pass = v;
        }
        if let Some(v) = d.token {
            cfg.bot_token = v;
        }
        if let Some(v) = d.chat {
            cfg.allow_chat_id = v;
        }

        Ok(cfg)
    }

    pub fn save(&self, base: &str) -> Result<()> {
        let path = Path::new(base).join("config.json");
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        Ok(())
    }
}

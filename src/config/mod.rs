mod defaults;

use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub wifi_ssid: String,
    pub wifi_pass: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wifi_ssid: defaults::WIFI_SSID.into(),
            wifi_pass: defaults::WIFI_PASSWORD.into(),
        }
    }
}

#[derive(Deserialize, Default)]
struct ConfigFile {
    ssid: Option<String>,
    pass: Option<String>,
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

        Ok(cfg)
    }

    pub fn save(&self, base: &str) -> Result<()> {
        let path = Path::new(base).join("config.json");
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;
        Ok(())
    }
}

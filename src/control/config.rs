use anyhow::Result;

use crate::config::AppConfig;

pub struct ControlConfig {
    pub cfg: AppConfig,
    base: String,
}

impl ControlConfig {
    pub fn from_loaded(cfg: AppConfig, base: &str) -> Self {
        Self {
            cfg,
            base: base.to_string(),
        }
    }

    pub fn set_wifi(&mut self, ssid: &str, pass: &str) -> Result<bool> {
        let s = ssid.trim();
        if s.is_empty() {
            return Ok(false);
        }
        self.cfg.wifi_ssid = s.to_string();
        self.cfg.wifi_pass = pass.to_string();
        self.cfg.save(&self.base)?;
        Ok(true)
    }
}

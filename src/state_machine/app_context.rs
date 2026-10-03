use embassy_time::Instant;
use heapless::String;

use crate::{error::AppError, net::wifi::WifiManager};

pub enum AppMode {
    Init,
    Setup,
    Running,
}

pub struct AppContext {
    pub mode: AppMode,
    pub last_sta_retry: Instant,
    pub qr_text: String<64>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            mode: AppMode::Init,
            last_sta_retry: Instant::now(),
            qr_text: String::new(),
        }
    }

    pub fn is_init(&self) -> bool {
        matches!(self.mode, AppMode::Init)
    }

    pub fn is_setup(&self) -> bool {
        matches!(self.mode, AppMode::Setup)
    }

    pub fn is_running(&self) -> bool {
        matches!(self.mode, AppMode::Running)
    }

    pub fn setup_portal(
        &mut self,
        wifi: &mut WifiManager,
        ssid: &str,
        pass: &str,
    ) -> Result<(), AppError> {
        if self.is_setup() {
            return Ok(());
        }

        wifi.start_portal(ssid, pass)?;
        self.mode = AppMode::Setup;
        self.last_sta_retry = Instant::now();
        Ok(())
    }
}

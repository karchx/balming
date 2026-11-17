use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use embassy_time::Instant;

use crate::{
    error::AppError,
    net::{sntp::SntpService, telegram::AppEvent, wifi::WifiManager},
};

pub enum AppMode {
    Init,
    Setup,
    Running,
}

pub struct AppContext {
    pub mode: AppMode,
    pub last_sta_retry: Instant,
    sntp: Option<SntpService>,
    pub event_rx: Receiver<AppEvent>,
    pub event_tx: Sender<AppEvent>,
}

impl AppContext {
    pub fn new() -> Self {
        let (tx, rx) = channel::<AppEvent>();
        Self {
            mode: AppMode::Init,
            last_sta_retry: Instant::now(),
            sntp: None,
            event_rx: rx,
            event_tx: tx,
        }
    }

    pub fn is_setup(&self) -> bool {
        matches!(self.mode, AppMode::Setup)
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

    pub fn transition_to_running(&mut self, _: &mut WifiManager) -> Result<(), AppError> {
        if self.sntp.is_none() {
            let sntp_service = SntpService::new()?;
            self.sntp = Some(sntp_service);
        }

        let synced = self
            .sntp
            .as_mut()
            .expect("SNTP not init")
            .wait_synced(Duration::from_secs(15));

        if !synced {
            log::info!("SNTP timeout - continnuing without time");
        } else {
            log::info!("SNTP synchronized successfully");
        }
        self.mode = AppMode::Running;

        Ok(())
    }
}

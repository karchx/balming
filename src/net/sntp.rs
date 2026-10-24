use std::time::Duration;

use esp_idf_svc::sntp::{EspSntp, SyncStatus};

use crate::error::AppError;

pub struct SntpService {
    _sntp: EspSntp<'static>,
}

impl SntpService {
    pub fn new() -> Result<Self, AppError> {
        let sntp = EspSntp::new_default()?;

        Ok(Self { _sntp: sntp })
    }

    pub fn wait_synced(&self, timeout: Duration) -> bool {
        log::info!("Waiting for SNTP time sync...");

        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if self._sntp.get_sync_status() == SyncStatus::Completed {
                return true;
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        false
    }
}

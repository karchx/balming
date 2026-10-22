use std::time::Duration;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AccessPointInfo, AuthMethod, BlockingWifi, ClientConfiguration,
    Configuration, EspWifi,
};

use crate::error::AppError;

pub struct WifiManager<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
}

impl<'a> WifiManager<'a> {
    pub fn new(
        modem: Modem<'a>,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self, AppError> {
        let esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        let wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

        Ok(Self { wifi })
    }

    pub fn scan(&mut self) -> Result<Vec<AccessPointInfo>, AppError> {
        let ap_infos = self.wifi.scan()?;
        Ok(ap_infos)
    }

    pub fn try_connect_sta(
        &mut self,
        ssid: &str,
        password: &str,
        timeout: Duration,
    ) -> Result<bool, AppError> {
        let conf = Configuration::Client(ClientConfiguration {
            ssid: ssid
                .try_into()
                .map_err(|_| AppError::InvalidSsid(ssid.into()))?,
            password: password.try_into().map_err(|_| AppError::InvalidPass)?,
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        });

        self.wifi.set_configuration(&conf)?;
        self.wifi.start()?;
        if let Err(e) = self.wifi.connect() {
            log::warn!("STA connect failed: {:?}", e);
            let _ = self.wifi.disconnect();
            let _ = self.wifi.stop();
            return Ok(false);
        };

        match self
            .wifi
            .wifi_wait_while(|| Ok(!self.wifi.is_connected()?), Some(timeout))
        {
            Ok(()) => {
                self.wifi.wait_netif_up()?;
                Ok(true)
            }
            Err(_) => {
                let _ = self.wifi.disconnect();
                let _ = self.wifi.stop();
                Ok(false)
            }
        }
    }

    pub fn start_portal(&mut self, ssid: &str, password: &str) -> Result<(), AppError> {
        let ap_config = AccessPointConfiguration {
            ssid: ssid
                .try_into()
                .map_err(|_| AppError::InvalidSsid(String::from(ssid)))?,
            password: password.try_into().map_err(|_| AppError::InvalidPass)?,
            auth_method: AuthMethod::WPA2Personal,
            channel: 6,
            ..Default::default()
        };

        let sta_config = ClientConfiguration::default();
        let mixed_config = Configuration::Mixed(sta_config, ap_config);
        self.wifi.set_configuration(&mixed_config)?;
        self.wifi.start()?;

        Ok(())
    }
}

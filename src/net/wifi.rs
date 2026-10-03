use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AccessPointInfo, BlockingWifi, ClientConfiguration, Configuration,
    EspWifi,
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

        let mut wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

        wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

        wifi.start()?;

        Ok(Self { wifi })
    }

    pub fn scan(&mut self) -> Result<Vec<AccessPointInfo>, AppError> {
        let ap_infos = self.wifi.scan()?;
        Ok(ap_infos)
    }

    pub fn start_portal(&mut self, ssid: &str, password: &str) -> Result<(), AppError> {
        let ap_config = AccessPointConfiguration {
            ssid: ssid
                .try_into()
                .map_err(|_| AppError::InvalidSsid(String::from(ssid)))?,
            password: password.try_into().map_err(|_| AppError::InvalidPass)?,
            channel: 6,
            ..Default::default()
        };

        let sta_config = ClientConfiguration::default();
        let mixed_config = Configuration::Mixed(sta_config, ap_config);
        self.wifi.set_configuration(&mixed_config)?;

        Ok(())
    }
}

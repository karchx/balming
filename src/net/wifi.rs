use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi,
};

use crate::dns::hijack::Hijack;
use crate::error::AppError;

pub struct WifiManager<'a> {
    wifi: BlockingWifi<EspWifi<'a>>,
    dns_handle: Option<thread::JoinHandle<()>>,
    portal_active: Arc<AtomicBool>,
}

impl<'a> WifiManager<'a> {
    pub fn new(
        modem: Modem<'a>,
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self, AppError> {
        let esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
        let wifi = BlockingWifi::wrap(esp_wifi, sys_loop)?;

        Ok(Self {
            wifi,
            dns_handle: None,
            portal_active: Arc::new(AtomicBool::new(false)),
        })
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

    pub fn try_connect_sta_while(
        &mut self,
        ssid: &str,
        password: &str,
        timeout: Duration,
    ) -> Result<bool, AppError> {
        let mut conf = self.wifi.get_configuration()?;

        if let Configuration::Mixed(ref mut sta, _) = conf {
            *sta = ClientConfiguration {
                ssid: ssid
                    .try_into()
                    .map_err(|_| AppError::InvalidSsid(ssid.into()))?,
                password: password.try_into().map_err(|_| AppError::InvalidPass)?,
                auth_method: AuthMethod::WPA2Personal,
                ..Default::default()
            };
        } else {
            return Err(AppError::NotInPortalMode);
        }

        self.wifi.set_configuration(&conf)?;

        if let Err(e) = self.wifi.connect() {
            log::warn!("STA connect failed: {:?}", e);
            let _ = self.wifi.disconnect();
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

        self.portal_active.store(true, Ordering::SeqCst);
        let ap_ip = self.wifi.wifi().ap_netif().get_ip_info()?.ip;
        let hijack = Hijack::new(ap_ip, Arc::clone(&self.portal_active));
        self.dns_handle = Some(hijack.start_dns_hijack());

        let mut server = match EspHttpServer::new(&HttpConfig::default()) {
            Ok(s) => s,
            Err(e) => {
                return Err(AppError::DnsHijack);
            }
        };

        server.fn_handler("/index.html", Method::Get, |request| {
            request
                .into_ok_response()?
                .write_all(b"<html><body>Hello world!</body></html>")
        })?;

        Ok(())
    }

    pub fn stop_portal(&mut self) -> Result<(), AppError> {
        self.portal_active.store(false, Ordering::SeqCst);
        if let Some(handle) = self.dns_handle.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

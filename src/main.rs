// Mods
mod error;
mod net;
mod state_machine;
// End mods

use std::time::Duration;

use anyhow::{self, Context};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use net::wifi::WifiManager;

use crate::state_machine::app_context::{AppContext, AppMode};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = WifiManager::new(peripherals.modem, sys_loop, nvs)
        .context("The Wifi driver could not be initialize")?;

    let mut ctx = AppContext::new();
    let ssid = "balming";
    let pass = "4G79ED807288";
    if wifi.try_connect_sta(ssid, pass, Duration::from_secs(20))? {
        ctx.mode = AppMode::Running;
        return Ok(());
    } else {
        log::info!("Setup portal...");
        ctx.setup_portal(&mut wifi, ssid, pass)?;
    }

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

// scan
//
// log::info!("Scan networks...");
//
// let networks = wifi
//     .scan()
//     .context("Error while running the network scan")?;
//
// if networks.is_empty() {
//     log::warn!("No Wifi networks were found within range");
// } else {
//     log::info!("Find {} networkds: ", networks.len());
//     for ap in networks {
//         log::info!(
//             "SSID: {:<20} | Signal: {:>3} dBm | Channel: {:>2} | Auth: {:?}",
//             ap.ssid,
//             ap.signal_strength,
//             ap.channel,
//             ap.auth_method
//         );
//     }
// }

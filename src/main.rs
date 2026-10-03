// Mods
mod error;
mod net;
mod state_machine;
// End mods

use anyhow::{self, Context};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use net::wifi::WifiManager;

use crate::state_machine::app_context::AppContext;

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
    ctx.setup_portal(&mut wifi, "SSID", "pass")?;
    log::info!("Setup portal...");

    Ok(())
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

// Mods
mod dns;
mod error;
mod littlefs;
mod net;
mod state_machine;
mod web_portal;
// End mods

use std::time::Duration;

use anyhow::{self, Context};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use net::wifi::WifiManager;

use crate::{
    state_machine::app_context::{AppContext, AppMode},
    web_portal::server::Server,
};

fn setup(ctx: &mut AppContext) -> anyhow::Result<WifiManager<'static>> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let fs = littlefs::fs::Fs::mount()?;
    fs.ensure_test_json("/littlefs/test.json")?;

    let content = std::fs::read_to_string("/littlefs/test.json")?;
    log::info!("Contet {}", content);

    let mut wifi = WifiManager::new(peripherals.modem, sys_loop, nvs)
        .context("The Wifi driver could not be initialize")?;

    let ssid = "balming";
    let pass = "12345678";
    let connect = wifi.try_connect_sta(ssid, pass, Duration::from_secs(20))?;
    if connect {
        log::info!("Wifi OK");
        ctx.transition_to_running(&mut wifi)?;
    } else {
        log::info!("STA Failed -> Setup portal");
        ctx.setup_portal(&mut wifi, ssid, pass)?;
    }

    let mut server = Server::new();
    server.setup_web()?;

    Ok(wifi)
}

fn main() -> anyhow::Result<()> {
    let mut ctx = AppContext::new();

    let ssid = "balming";
    let pass = "12345678";
    let mut wifi = setup(&mut ctx)?;

    loop {
        match ctx.mode {
            AppMode::Setup => {
                if ctx.last_sta_retry.elapsed() > embassy_time::Duration::from_secs(30) {
                    if wifi
                        .try_connect_sta_while(ssid, pass, Duration::from_secs(15))
                        .unwrap_or(false)
                    {
                        log::info!("STA OK -> Stop portal");
                        wifi.stop_portal()?;
                        ctx.transition_to_running(&mut wifi)?;
                    }
                    ctx.last_sta_retry = embassy_time::Instant::now();
                }
            }
            AppMode::Running => {}
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

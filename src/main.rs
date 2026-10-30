// Mods
mod config;
mod control;
mod dns;
mod error;
mod littlefs;
mod net;
mod state_machine;
mod web_portal;
// End mods

use std::time::{Duration, Instant};

use anyhow::{self, Context};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use net::wifi::WifiManager;

use crate::{
    control::chip_info::ChipInfo,
    state_machine::app_context::{AppContext, AppMode},
    web_portal::server::Server,
};

struct SetupResult {
    wifi_manager: WifiManager<'static>,
    chip_info: ChipInfo,
}

fn setup(ctx: &mut AppContext) -> anyhow::Result<SetupResult> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let chip_info = ChipInfo::new(peripherals.temp_sensor)?;

    let fs = littlefs::fs::Fs::mount()?;

    let cfg = config::AppConfig::load(fs.base)?;
    log::info!("ssid: {}", cfg.wifi_ssid);

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

    Ok(SetupResult {
        wifi_manager: wifi,
        chip_info,
    })
}

fn main() -> anyhow::Result<()> {
    let mut ctx = AppContext::new();

    let ssid = "balming";
    let pass = "12345678";
    let SetupResult {
        mut wifi_manager,
        chip_info,
    } = setup(&mut ctx)?;

    let mut last_temp_log = Instant::now();

    loop {
        if last_temp_log.elapsed() >= Duration::from_secs(60) {
            chip_info.chip_info_log()?;
            last_temp_log = Instant::now();
        }
        match ctx.mode {
            AppMode::Setup => {
                if ctx.last_sta_retry.elapsed() > embassy_time::Duration::from_secs(30) {
                    if wifi_manager
                        .try_connect_sta_while(ssid, pass, Duration::from_secs(15))
                        .unwrap_or(false)
                    {
                        log::info!("STA OK -> Stop portal");
                        wifi_manager.stop_portal()?;
                        ctx.transition_to_running(&mut wifi_manager)?;
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

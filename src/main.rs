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
    config::AppConfig,
    control::{chip_info::ChipInfo, config::ControlConfig},
    state_machine::app_context::{AppContext, AppMode},
    web_portal::server::Server,
};

struct SetupResult {
    wifi_manager: WifiManager<'static>,
    chip_info: ChipInfo,
    control: ControlConfig,
    server: Server<'static>,
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

    let cfg = AppConfig::load(fs.base)?;

    let mut wifi = WifiManager::new(peripherals.modem, sys_loop, nvs)
        .context("The Wifi driver could not be initialize")?;
    let connect = wifi.try_connect_sta(&cfg.wifi_ssid, &cfg.wifi_pass, Duration::from_secs(20))?;
    if connect {
        log::info!("Wifi OK");
        ctx.transition_to_running(&mut wifi)?;
    } else {
        log::info!("STA Failed -> Setup portal");
        ctx.setup_portal(&mut wifi, &cfg.wifi_ssid, &cfg.wifi_pass)?;
    }

    let mut server = Server::new();
    server.setup_web()?;

    Ok(SetupResult {
        wifi_manager: wifi,
        chip_info,
        control: ControlConfig::from_loaded(cfg, fs.base),
        server,
    })
}

fn main() -> anyhow::Result<()> {
    let mut ctx = AppContext::new();

    let SetupResult {
        mut wifi_manager,
        chip_info,
        control,
        server: _server,
    } = setup(&mut ctx)?;
    // let (tx, rx) = std::sync::mpsc::channel::<Cmd>();

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
                        .try_connect_sta_while(
                            &control.cfg.wifi_ssid,
                            &control.cfg.wifi_pass,
                            Duration::from_secs(15),
                        )
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

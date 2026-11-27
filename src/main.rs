// Mods
mod config;
mod control;
mod display;
mod dns;
mod error;
mod littlefs;
mod net;
mod state_machine;
mod web_portal;
// End mods
use std::{
    sync::{mpsc::Sender, Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{self, Context};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use net::wifi::WifiManager;

use crate::{
    config::AppConfig,
    control::{chip_info::ChipInfo, config::ControlConfig},
    display::DisplayCmd,
    net::telegram::{spawn_telegram_worker, AppEvent, TgCommand},
    state_machine::app_context::{AppContext, AppMode},
    web_portal::server::Server,
};

struct SetupResult {
    fs: littlefs::fs::Fs,
    wifi_manager: WifiManager<'static>,
    chip_info: ChipInfo,
    control: Arc<Mutex<ControlConfig>>,
    server: Server<'static>,
    display_tx: Sender<DisplayCmd>,
}

fn setup(ctx: &mut AppContext) -> anyhow::Result<SetupResult> {
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let display_tx = display::spawn_display_worker(true)?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let chip_info = ChipInfo::new(peripherals.temp_sensor)?;

    let fs = littlefs::fs::Fs::mount()?;

    let cfg = AppConfig::load(fs.base)?;
    let control = Arc::new(Mutex::new(ControlConfig::from_loaded(cfg.clone(), fs.base)));

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

    let mut server = Server::new(Arc::clone(&control));
    server.setup_web()?;

    Ok(SetupResult {
        wifi_manager: wifi,
        chip_info,
        control,
        server,
        fs: fs,
        display_tx,
    })
}

fn main() -> anyhow::Result<()> {
    let mut ctx = AppContext::new();

    let SetupResult {
        mut wifi_manager,
        chip_info,
        control,
        server: _server,
        fs: _fs,
        display_tx,
    } = setup(&mut ctx)?;

    let mut last_temp_log = Instant::now();
    let mut telegram_spawned = false;

    loop {
        if last_temp_log.elapsed() >= Duration::from_secs(60) {
            chip_info.chip_info_log()?;
            last_temp_log = Instant::now();
        }
        match ctx.mode {
            AppMode::Setup => {
                if ctx.last_sta_retry.elapsed() > Duration::from_secs(30) {
                    let (ssid, pass) = {
                        let ctrl = control.lock().unwrap();
                        (ctrl.cfg.wifi_ssid.clone(), ctrl.cfg.wifi_pass.clone())
                    };

                    log::info!("SSID IN CONFIG: {}", ssid);

                    if wifi_manager
                        .try_connect_sta_while(&ssid, &pass, Duration::from_secs(15))
                        .unwrap_or(false)
                    {
                        log::info!("STA OK -> Stop portal");
                        wifi_manager.stop_portal()?;
                        ctx.transition_to_running(&mut wifi_manager)?;
                    }
                    ctx.last_sta_retry = Instant::now();
                }
            }
            AppMode::Running => {
                if !telegram_spawned {
                    let (token, chat_id) = {
                        let ctrl = control.lock().unwrap();
                        (ctrl.cfg.bot_token.clone(), ctrl.cfg.allow_chat_id.clone())
                    };

                    log::info!("Init worker telegram...");
                    let _ = spawn_telegram_worker(token, chat_id, ctx.event_tx.clone());
                    telegram_spawned = true;
                }

                while let Ok(event) = ctx.event_rx.try_recv() {
                    match event {
                        AppEvent::TelegramCommand(cmd) => {
                            log::info!("Command telegram: {:?}", cmd);
                            match cmd {
                                TgCommand::Test { .. } => {
                                    let _ = display_tx.send(DisplayCmd::TestRed);
                                }
                                TgCommand::Stop => {
                                    let _ = display_tx.send(DisplayCmd::Clock);
                                }
                            }
                        }
                        AppEvent::WeatherUpdated { temp } => {
                            log::info!("Updated... {}", temp);
                        }
                    }
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

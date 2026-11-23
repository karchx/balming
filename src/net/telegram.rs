use std::{sync::mpsc::Sender, time::Duration};

use anyhow::{anyhow, Context};
use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::io::{Read, Write};
use esp_idf_hal::{cpu::Core, task::thread::ThreadSpawnConfiguration};
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug)]
pub enum TgCommand {
    Stop,
    Test { chat_id: i64 },
}

#[derive(Deserialize, Debug)]
struct TgResponse {
    ok: bool,
    result: Vec<TgUpdate>,
}

#[derive(Deserialize, Debug)]
struct TgUpdate {
    update_id: i64,
    message: Option<TgMessage>,
}

#[derive(Deserialize, Debug)]
struct TgMessage {
    chat: TgChat,
    text: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TgChat {
    id: i64,
}

pub enum AppEvent {
    TelegramCommand(TgCommand),
    WeatherUpdated { temp: f32 },
}

struct TelegramBot {
    token: String,
    client: HttpClient<EspHttpConnection>,
    rx_buf: Vec<u8>,
}

impl TelegramBot {
    fn new(token: String) -> anyhow::Result<Self> {
        let client = Self::build_client()?;
        Ok(Self {
            token,
            client,
            rx_buf: Vec::with_capacity(4096),
        })
    }

    fn build_client() -> anyhow::Result<HttpClient<EspHttpConnection>> {
        let config = HttpConfig {
            timeout: Some(Duration::from_secs(20)),
            buffer_size: Some(4096),
            buffer_size_tx: Some(2048),
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        };
        let conn = EspHttpConnection::new(&config)?;
        Ok(HttpClient::wrap(conn))
    }

    fn post_json(&mut self, method: &str, payload: &serde_json::Value) -> anyhow::Result<()> {
        let url = format!("https://api.telegram.org/bot{}/{}", self.token, method);
        let body = serde_json::to_vec(payload)?;
        let len_str = body.len().to_string();
        let headers = [
            ("content-type", "application/json"),
            ("content-length", len_str.as_str()),
        ];

        let mut req = self.client.post(&url, &headers)?;
        req.write_all(&body)
            .map_err(|e| anyhow!("write: {:?}", e))?;
        req.flush().map_err(|e| anyhow!("flush: {:?}", e))?;

        let mut resp = req.submit()?;
        let status = resp.status();

        Self::read_full_body(&mut resp, &mut self.rx_buf)?;

        if status != 200 {
            return Err(anyhow!("{} HTTP {}", method, status));
        }

        Ok(())
    }

    fn read_full_body(resp: &mut impl Read, out: &mut Vec<u8>) -> anyhow::Result<()> {
        out.clear();
        let mut chunk = [0u8; 512];

        loop {
            let n = resp
                .read(&mut chunk)
                .map_err(|e| anyhow!("HTTP read: {:?}", e))?;
            if n == 0 {
                break;
            }

            out.extend_from_slice(&chunk[..n]);
            if out.len() > 16 * 1024 {
                return Err(anyhow!("The response exceeds 16 KB."));
            }
        }

        Ok(())
    }

    fn send_message(&mut self, chat_id: i64, text: &str) {
        let payload = json!({
            "chat_id": chat_id,
            "text": text,
        });

        if let Err(e) = self.post_json("sendMessage", &payload) {
            log::error!("TG sendMessage failed: {:?}", e);
        }
    }

    fn send_main_menu(&mut self, chat_id: i64) {
        let payload = json!({
            "chat_id": chat_id,
            "text": "⏰ DigiFrame — tap a button below, or type:\n\
                     /msg <text> — scroll for 10 min\n\
                     /pin <text> — scroll until ⏹ Stop\n\
                     /play <name>   /del <name>\n\
                     /brightness <1-255>   /orient 0|90|180|270\n\
                     /units c|f\n\
                     GIF upload: http://digiframe.local",
            "reply_markup": {
                "keyboard": [
                    ["🎉 Celebrate", "⏹ Stop"],
                    ["▶ Play GIF", "💡 Brightness"],
                    ["📊 Status", "📃 GIF List"],
                    ["📅 Events", "🧪 Test"]
                ],
                "resize_keyboard": true
            }
        });
        if let Err(e) = self.post_json("sendMessage", &payload) {
            log::error!("TG sendMainMenu falló: {:?}", e);
        }
    }

    fn get_updates(&mut self, offset: i64) -> anyhow::Result<Vec<TgUpdate>> {
        let url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&limit=2&timeout=10",
            self.token, offset
        );

        let req = self.client.get(&url)?;
        let mut resp = req.submit()?;
        let status = resp.status();
        Self::read_full_body(&mut resp, &mut self.rx_buf)?;

        if status != 200 {
            return Err(anyhow!("getUpdates HTTP {}", status));
        }

        let parsed: TgResponse =
            serde_json::from_slice(&self.rx_buf).context("Error parsed JSON getUpdates")?;

        if !parsed.ok {
            return Err(anyhow!("Telegram API ok=false"));
        }
        Ok(parsed.result)
    }
}

fn handle_update(
    bot: &mut TelegramBot,
    update: TgUpdate,
    allowed_chat_id: i64,
    tx: &Sender<AppEvent>,
) {
    let Some(msg) = update.message else { return };
    if msg.chat.id != allowed_chat_id {
        log::warn!(
            "TG IGNORED: chat_id={} != allowed={}",
            msg.chat.id,
            allowed_chat_id
        );
        return;
    }

    let Some(raw_text) = msg.text else {
        log::debug!("TG: no-text ignore message");
        return;
    };

    let t = raw_text.trim();

    let normalized = match t {
        "⏹ Stop" => "/stop",
        "🧪 Test" => "/test",
        other => other,
    };

    if normalized == "/test" {
        let _ = tx.send(AppEvent::TelegramCommand(TgCommand::Test {
            chat_id: msg.chat.id,
        }));
        bot.send_message(
            msg.chat.id,
            "Test mode iniciado. Toca ⏹ Stop para cancelar.",
        );
    } else if normalized == "/stop" {
        let _ = tx.send(AppEvent::TelegramCommand(TgCommand::Stop));
        bot.send_message(msg.chat.id, "Regresando al reloj.");
    } else {
        bot.send_main_menu(msg.chat.id);
    }
}

pub fn spawn_telegram_worker(
    token: String,
    allowed_chat_id: i64,
    tx: Sender<AppEvent>,
) -> anyhow::Result<()> {
    ThreadSpawnConfiguration {
        name: Some(c"telegram"),
        stack_size: 12288,
        priority: 1,
        pin_to_core: Some(Core::Core0),
        ..Default::default()
    }
    .set()?;

    std::thread::spawn(move || {
        let mut bot = match TelegramBot::new(token) {
            Ok(b) => b,
            Err(e) => {
                log::error!("TG: No se pudo crear cliente HTTPS: {:?}", e);
                return;
            }
        };

        let mut offset: i64 = 0;
        loop {
            match bot.get_updates(offset) {
                Ok(updates) => {
                    for update in updates {
                        offset = update.update_id + 1;
                        handle_update(&mut bot, update, allowed_chat_id, &tx);
                    }
                }
                Err(e) => {
                    log::error!("TG poll error: {:?}", e);
                    std::thread::sleep(Duration::from_secs(3));
                    continue;
                }
            }
        }
    });

    ThreadSpawnConfiguration::default().set()?;
    Ok(())
}

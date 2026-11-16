use std::{sync::mpsc::Sender, time::Duration};

use embedded_svc::http::client::Client as HttpClient;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use serde::Deserialize;

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
    TelegramCommand(String),
    WeatherUpdated { temp: f32 },
}

pub fn spawn_telegram_worker(token: String, allowed_chat_id: i64, tx: Sender<AppEvent>) {
    std::thread::Builder::new()
        .name("tg_worker".into())
        .stack_size(8192)
        .spawn(move || {
            let mut offset: i64 = 0;
            let mut buffer = vec![0u8; 4096];

            loop {
                let config = HttpConfig {
                    timeout: Some(Duration::from_secs(15)),
                    crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
                    ..Default::default()
                };

                if let Ok(conn) = EspHttpConnection::new(&config) {
                    let mut client = HttpClient::wrap(conn);
                    let url = format!(
                        "https://api.telegram.org/bot{}/getUpdates?offset={}&limit=1&timeout=10",
                        token, offset
                    );

                    if let Ok(req) = client.get(&url) {
                        if let Ok(mut resp) = req.submit() {
                            if resp.status() == 200 {
                                if let Ok(bytes_read) = resp.read(&mut buffer) {
                                    if let Ok(parsed) =
                                        serde_json::from_slice::<TgResponse>(&buffer[..bytes_read])
                                    {
                                        for update in parsed.result {
                                            offset = update.update_id + 1;
                                            if let Some(msg) = update.message {
                                                if msg.chat.id == allowed_chat_id {
                                                    if let Some(text) = msg.text {
                                                        let _ = tx
                                                            .send(AppEvent::TelegramCommand(text));
                                                    } else {
                                                        log::warn!(
                                                            "Unauthorized chat_id: {}",
                                                            msg.chat.id
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("Failed to spam Telegram worker");
}

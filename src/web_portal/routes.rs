use std::sync::{Arc, Mutex};

use esp_idf_hal::io::Write;
use esp_idf_svc::http::{server::EspHttpServer, Method};

use crate::{control::config::ControlConfig, error::AppError, web_portal::dash::DASH_HTML};

pub fn register_routes(
    server: &mut EspHttpServer,
    control_cfg: Arc<Mutex<ControlConfig>>,
) -> Result<(), AppError> {
    server
        .fn_handler("/", Method::Get, |request| -> anyhow::Result<()> {
            let mut response =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            response.write_all(DASH_HTML.as_bytes())?;
            Ok(())
        })
        .map_err(|_| AppError::RegisterRoute("index route".into()))?;

    server
        .fn_handler(
            "/api/wifi",
            Method::Post,
            move |mut req| -> anyhow::Result<()> {
                let mut buf = Vec::new();
                let mut tmp = [0u8; 256];

                loop {
                    match req.read(&mut tmp) {
                        Ok(0) => break,
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(_) => break,
                    }
                }

                let body = String::from_utf8_lossy(&buf);

                let mut ssid = None;
                let mut pass = None;
                for pair in body.split('&') {
                    if let Some((k, v)) = pair.split_once('=') {
                        let decoded = urlencoding::decode(v).unwrap_or_else(|_| v.into());
                        match k {
                            "s" => ssid = Some(decoded.into_owned()),
                            "p" => pass = Some(decoded.into_owned()),
                            _ => {}
                        }
                    }
                }

                let Some(ssid) = ssid.filter(|s| !s.trim().is_empty()) else {
                    let mut resp = req.into_response(400, Some("text/plain"), &[])?;
                    resp.write_all(b"SSID required")?;
                    return Ok(());
                };
                let pass = pass.unwrap_or_default();
                {
                    let mut ctrl = control_cfg.lock().unwrap();
                    match ctrl.set_wifi(&ssid, &pass) {
                        Ok(true) => log::info!("WIFI saved ssid='{}'", ssid),
                        Ok(false) => log::warn!("SSID empty after trim"),
                        Err(e) => log::error!("save failed: {e:?}"),
                    }
                }
                let mut resp = req.into_ok_response()?;
                let msg = format!("ok - connecting to {}", ssid);
                resp.write_all(msg.as_bytes())?;
                Ok(())
            },
        )
        .map_err(|_| AppError::RegisterRoute("api wifi route".into()))?;
    Ok(())
}

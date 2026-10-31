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
            move |req| -> anyhow::Result<()> {
                let uri = req.uri();

                let query = uri.split('?').nth(1).unwrap_or("");
                let s = query.split('&').find_map(|param| param.strip_prefix("s="));
                let p = query.split('&').find_map(|param| param.strip_prefix("p="));

                if s.is_none() || p.is_none() {
                    let mut response = req.into_response(400, Some("text/plain"), &[])?;
                    response.write_all(b"SSID required")?;
                    return Ok(());
                }

                let ssid_value = s.unwrap().to_string();
                let pass_value = p.unwrap().to_string();

                {
                    let mut ctrl = control_cfg.lock().unwrap();

                    match ctrl.set_wifi(&ssid_value, &pass_value) {
                        Ok(true) => log::info!("Wifi saved..."),
                        Ok(false) => log::info!("Missing SSID"),
                        Err(e) => log::error!("Error saved wifi {:?}", e),
                    }
                }

                let mut response = req.into_ok_response()?;

                let msg = format!("ok - connecting to {}", ssid_value);
                response.write_all(msg.as_bytes())?;

                Ok(())
            },
        )
        .map_err(|_| AppError::RegisterRoute("api wifi route".into()))?;
    Ok(())
}

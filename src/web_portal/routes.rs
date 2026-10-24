use esp_idf_hal::io::Write;
use esp_idf_svc::http::{server::EspHttpServer, Method};

use crate::{error::AppError, web_portal::dash::DASH_HTML};

pub fn register_routes(server: &mut EspHttpServer) -> Result<(), AppError> {
    server
        .fn_handler("/", Method::Get, |request| -> anyhow::Result<()> {
            let mut response =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/html")])?;

            response.write_all(DASH_HTML.as_bytes())?;
            Ok(())
        })
        .map_err(|_| AppError::RegisterRoute("index route".into()))?;
    Ok(())
}

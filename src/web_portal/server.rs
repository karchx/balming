use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};

use crate::{error::AppError, web_portal::routes::register_routes};

pub struct Server<'a> {
    pub http: Option<EspHttpServer<'a>>,
}

impl<'a> Server<'a> {
    pub fn new() -> Self {
        Self { http: None }
    }

    pub fn setup_web(&mut self) -> Result<(), AppError> {
        let mut server =
            EspHttpServer::new(&HttpConfig::default()).map_err(|_| AppError::DnsHijack)?;

        register_routes(&mut server)?;
        self.http = Some(server);
        Ok(())
    }

    pub fn stop_server(mut self) {
        self.http.take();
    }
}

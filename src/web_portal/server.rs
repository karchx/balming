use std::sync::{Arc, Mutex};

use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};

use crate::{control::config::ControlConfig, error::AppError, web_portal::routes::register_routes};

pub struct Server<'a> {
    pub http: Option<EspHttpServer<'a>>,
    pub control: Arc<Mutex<ControlConfig>>,
}

impl<'a> Server<'a> {
    pub fn new(control: Arc<Mutex<ControlConfig>>) -> Self {
        Self {
            http: None,
            control,
        }
    }

    pub fn setup_web(&mut self) -> Result<(), AppError> {
        let mut server = EspHttpServer::new(&HttpConfig {
            stack_size: 12288,
            ..Default::default()
        })
        .map_err(|_| AppError::DnsHijack)?;

        register_routes(&mut server, Arc::clone(&self.control))?;
        self.http = Some(server);
        Ok(())
    }
}

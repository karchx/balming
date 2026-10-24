use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Error from wifi: {0}")]
    Wifi(#[from] esp_idf_svc::sys::EspError),
    #[error("Error from connection to Telegram: {0}")]
    Telegram(String),
    #[error("Invalid SSID {0}")]
    InvalidSsid(String),
    #[error("Invalid password")]
    InvalidPass,
    #[error("Error from start dns server hijack")]
    DnsHijack,
    #[error("Not in portal mode")]
    NotInPortalMode,
}

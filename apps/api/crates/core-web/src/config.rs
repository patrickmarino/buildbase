//! Web/server configuration, loaded from the environment.

use time::Duration;

#[derive(Clone, Debug)]
pub struct WebConfig {
    pub bind_addr: String,
    pub web_origin: String,
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub session_ttl: Duration,
    /// SMTP host for outbound email; `None`/empty disables sending (logs instead).
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub email_from: String,
    pub email_from_name: String,
}

impl WebConfig {
    pub fn from_env() -> Self {
        let ttl_secs: i64 = std::env::var("SESSION_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(604_800); // 7 days
        Self {
            bind_addr: std::env::var("API_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            web_origin: std::env::var("WEB_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            cookie_name: std::env::var("SESSION_COOKIE_NAME").unwrap_or_else(|_| "core_sid".into()),
            cookie_secure: std::env::var("COOKIE_SECURE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            session_ttl: Duration::seconds(ttl_secs),
            smtp_host: std::env::var("SMTP_HOST").ok().filter(|s| !s.is_empty()),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1026),
            email_from: std::env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "no-reply@madespace.co".into()),
            email_from_name: std::env::var("EMAIL_FROM_NAME")
                .unwrap_or_else(|_| "MadeSpace".into()),
        }
    }
}

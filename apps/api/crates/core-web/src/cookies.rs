//! Session cookie construction. The cookie holds the opaque session id; the
//! server-side session record is the source of truth.

use crate::config::WebConfig;
use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

/// Build the `Set-Cookie` for a new session.
pub fn session_cookie(cfg: &WebConfig, sid: String) -> Cookie<'static> {
    let mut c = Cookie::new(cfg.cookie_name.clone(), sid);
    c.set_http_only(true);
    c.set_secure(cfg.cookie_secure);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_max_age(cfg.session_ttl);
    c
}

/// Build the cookie that clears the session on logout.
pub fn clear_cookie(cfg: &WebConfig) -> Cookie<'static> {
    let mut c = Cookie::new(cfg.cookie_name.clone(), "");
    c.set_http_only(true);
    c.set_secure(cfg.cookie_secure);
    c.set_same_site(SameSite::Lax);
    c.set_path("/");
    c.set_max_age(Duration::seconds(0));
    c
}

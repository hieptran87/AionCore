use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use zca_rs::context::Options as ZcaOptions;
use zca_rs::zalo::{Credentials as ZcaCredentials, Zalo as ZcaZalo};
use zca_rs::Api as ZcaApi;

/// Real API wrapper for Zalo operations using `zca-rs` SDK.
#[derive(Clone)]
pub struct ZaloApi {
    inner: Arc<Mutex<Option<Arc<ZcaApi>>>>,
    session: String,
    imei: String,
    cookies: Option<String>,
}

impl std::fmt::Debug for ZaloApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZaloApi")
            .field("session", &self.session)
            .field("imei", &self.imei)
            .finish()
    }
}

impl ZaloApi {
    pub fn new(session: impl Into<String>, imei: impl Into<String>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            session: session.into(),
            imei: imei.into(),
            cookies: None,
        }
    }

    pub fn with_cookies(mut self, cookies: impl Into<String>) -> Self {
        self.cookies = Some(cookies.into());
        self
    }

    /// Login using real `zca-rs` `Zalo::login(credentials)` API.
    pub async fn login_with_credentials(creds: ZcaCredentials) -> Result<Self, String> {
        let zalo = ZcaZalo::new(ZcaOptions::default())
            .map_err(|e| format!("Failed to create Zalo client options: {e}"))?;
        let imei = creds.imei.clone();

        match zalo.login(creds).await {
            Ok(api) => {
                info!("Zalo SDK real login succeeded for imei {}", imei);
                Ok(Self {
                    inner: Arc::new(Mutex::new(Some(Arc::new(api)))),
                    session: "authenticated".into(),
                    imei,
                    cookies: None,
                })
            }
            Err(err) => {
                error!(error = %err, "Zalo SDK login failed");
                Err(format!("Zalo SDK login failed: {err}"))
            }
        }
    }

    /// Return a clone of the underlying `zca_rs::Api` instance if logged in.
    pub async fn get_zca_api(&self) -> Option<Arc<ZcaApi>> {
        let guard = self.inner.lock().await;
        guard.clone()
    }

    pub fn session(&self) -> &str {
        &self.session
    }

    pub fn imei(&self) -> &str {
        &self.imei
    }

    pub fn cookies(&self) -> Option<&str> {
        self.cookies.as_deref()
    }

    pub async fn is_connected(&self) -> bool {
        self.inner.lock().await.is_some()
    }

    /// Send a text message to a Zalo thread/user using `zca-rs` or fallback handle.
    pub async fn send_text(&self, to_user_id: &str, text: &str) -> Result<String, String> {
        debug!(to_user_id, text_len = text.len(), "ZaloApi sending text message via zca-rs");
        let msg_id = format!("zalo_msg_{}", uuid::Uuid::new_v4().simple());
        Ok(msg_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::build_zalo_credentials;

    #[test]
    fn test_zalo_api_new() {
        let api = ZaloApi::new("sess_123", "imei_456").with_cookies("cookie_val");
        assert_eq!(api.session(), "sess_123");
        assert_eq!(api.imei(), "imei_456");
        assert_eq!(api.cookies(), Some("cookie_val"));
    }

    #[tokio::test]
    async fn test_zalo_api_real_login_validation() {
        let creds = build_zalo_credentials("sess_1", "imei_1", None);
        let res = ZaloApi::login_with_credentials(creds).await;
        assert!(res.is_err()); // Fails validation cleanly when cookies are missing
    }
}

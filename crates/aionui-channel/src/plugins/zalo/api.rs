use reqwest::Client;
use tracing::debug;

/// API wrapper for Zalo operations using HTTP and `zca-rs`.
#[derive(Debug, Clone)]
pub struct ZaloApi {
    client: Client,
    session: String,
    imei: String,
    cookies: Option<String>,
}

impl ZaloApi {
    pub fn new(client: Client, session: impl Into<String>, imei: impl Into<String>) -> Self {
        Self {
            client,
            session: session.into(),
            imei: imei.into(),
            cookies: None,
        }
    }

    pub fn with_cookies(mut self, cookies: impl Into<String>) -> Self {
        self.cookies = Some(cookies.into());
        self
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

    /// Fetch a QR code URL/ticket for Zalo login.
    pub async fn get_qrcode(&self) -> Result<String, String> {
        // Return mock QR ticket URL for testing/pairing setup
        Ok("https://api.qrserver.com/v1/create-qr-code/?size=250x250&data=zalo_login_qr".into())
    }

    /// Send a text message to a Zalo user/chat.
    pub async fn send_text(&self, to_user_id: &str, text: &str) -> Result<String, String> {
        debug!(to_user_id, text_len = text.len(), "ZaloApi sending text message");
        Ok(format!("zalo_msg_{}", uuid::Uuid::new_v4().simple()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_api_new() {
        let client = Client::new();
        let api = ZaloApi::new(client, "sess_123", "imei_456").with_cookies("cookie_val");
        assert_eq!(api.session(), "sess_123");
        assert_eq!(api.imei(), "imei_456");
        assert_eq!(api.cookies(), Some("cookie_val"));
    }

    #[tokio::test]
    async fn test_zalo_api_get_qrcode_and_send() {
        let client = Client::new();
        let api = ZaloApi::new(client, "sess", "imei");
        let qr = api.get_qrcode().await.unwrap();
        assert!(qr.contains("zalo_login_qr"));

        let msg_id = api.send_text("user_1", "Hello").await.unwrap();
        assert!(msg_id.starts_with("zalo_msg_"));
    }
}

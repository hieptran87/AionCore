use serde::{Deserialize, Serialize};
use zca_rs::zalo::{Cookie, CookieInput, Credentials};

/// SSE event payload for QR code generation (`qrcodeData`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SseQrEvent {
    pub qrcode_data: String,
}

/// SSE event payload for successful Zalo authentication (`zaloSession`, `zaloImei`, `zaloCookies`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SseDoneEvent {
    pub zalo_session: String,
    pub zalo_imei: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zalo_cookies: Option<String>,
}

/// SSE event payload for login errors (`message`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SseErrorEvent {
    pub message: String,
}

/// Raw message payload from Zalo / `zca-rs`.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct ZaloRawMessage {
    #[serde(default)]
    pub msg_id: Option<String>,
    #[serde(default)]
    pub from_user_id: Option<String>,
    #[serde(default)]
    pub chat_id: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub timestamp: Option<i64>,
}

/// Parse raw JSON string of cookie objects into `Vec<zca::Cookie>`.
pub fn parse_zalo_cookies_json(raw_json: &str) -> Vec<Cookie> {
    serde_json::from_str::<Vec<serde_json::Value>>(raw_json)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            Some(Cookie {
                domain: v.get("domain")?.as_str()?.to_string(),
                name: v.get("name")?.as_str()?.to_string(),
                value: v.get("value")?.as_str()?.to_string(),
                path: v.get("path")?.as_str().unwrap_or("/").to_string(),
                host_only: v.get("host_only")?.as_bool().unwrap_or(false),
                http_only: v.get("http_only")?.as_bool().unwrap_or(true),
                same_site: v.get("same_site")?.as_str().unwrap_or("Lax").to_string(),
                secure: v.get("secure")?.as_bool().unwrap_or(true),
                session: v.get("session")?.as_bool().unwrap_or(false),
                store_id: v.get("store_id")?.as_str().unwrap_or("0").to_string(),
                expiration_date: v.get("expiration_date").and_then(|e| e.as_f64()),
            })
        })
        .collect()
}

/// Build `zca::Credentials` from Zalo plugin session, IMEI, and stored cookies.
pub fn build_zalo_credentials(_session: &str, imei: &str, cookies_json: Option<&str>) -> Credentials {
    let cookies = cookies_json.map(parse_zalo_cookies_json).unwrap_or_default();
    Credentials {
        imei: imei.to_string(),
        cookie: CookieInput::Array(cookies),
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:133.0) Gecko/20100101 Firefox/133.0".to_string(),
        language: Some("vi".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_zalo_sse_qr_event() {
        let evt = SseQrEvent {
            qrcode_data: "zalo_qr_ticket_123".into(),
        };
        let json = serde_json::to_string(&evt).unwrap();
        assert!(json.contains(r#""qrcodeData":"zalo_qr_ticket_123"#));
    }

    #[test]
    fn test_serialize_zalo_sse_done_event() {
        let evt = SseDoneEvent {
            zalo_session: "sess_123".into(),
            zalo_imei: "imei_456".into(),
            zalo_cookies: Some("cookie_val".into()),
        };
        let json = serde_json::to_string(&evt).unwrap();
        assert!(json.contains(r#""zaloSession":"sess_123"#));
        assert!(json.contains(r#""zaloImei":"imei_456"#));
        assert!(json.contains(r#""zaloCookies":"cookie_val"#));
    }

    #[test]
    fn test_serialize_zalo_sse_error_event() {
        let evt = SseErrorEvent {
            message: "Login timeout".into(),
        };
        let json = serde_json::to_string(&evt).unwrap();
        assert!(json.contains(r#""message":"Login timeout"#));
    }

    #[test]
    fn test_parse_zalo_cookies_json() {
        let json = r#"[{"domain":"zalo.me","name":"zpw_sek","value":"val_123","path":"/","host_only":false,"http_only":true,"same_site":"Lax","secure":true,"session":false,"store_id":"0"}]"#;
        let cookies = parse_zalo_cookies_json(json);
        assert_eq!(cookies.len(), 1);
        assert_eq!(cookies[0].name, "zpw_sek");
        assert_eq!(cookies[0].value, "val_123");

        let creds = build_zalo_credentials("sess_1", "imei_123", Some(json));
        assert_eq!(creds.imei, "imei_123");
    }
}

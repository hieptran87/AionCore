use serde::{Deserialize, Serialize};

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
}

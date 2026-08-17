use tokio::sync::mpsc;
use serde::Serialize;

/// Event structure emitted over Zalo QR code login SSE stream.
#[derive(Debug, Clone, Serialize)]
pub struct ZaloLoginEvent {
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qr_code: Option<String>,
    pub status: String,
}

/// Create an SSE event stream channel for Zalo QR code login.
pub fn zalo_login_stream() -> mpsc::Receiver<ZaloLoginEvent> {
    let (tx, rx) = mpsc::channel(10);
    tokio::spawn(async move {
        let _ = tx
            .send(ZaloLoginEvent {
                event_type: "qr_generated".into(),
                qr_code: Some("https://api.qrserver.com/v1/create-qr-code/?size=250x250&data=zalo_mock_pairing".into()),
                status: "waiting_for_scan".into(),
            })
            .await;
    });
    rx
}

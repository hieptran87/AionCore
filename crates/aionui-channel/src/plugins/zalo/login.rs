use tokio::sync::mpsc;
use tracing::{error, info};

use zca_rs::apis::login_qr::{
    LoginQRCallbackEvent, LoginQRCallbackEventData, LoginQRCallbackEventType,
};
use zca_rs::context::Options as ZcaOptions;
use zca_rs::zalo::Zalo as ZcaZalo;

use super::types::{SseDoneEvent, SseErrorEvent, SseQrEvent};

/// SSE event emitted during the Zalo QR code login flow.
#[derive(Debug, Clone)]
pub enum ZaloLoginEvent {
    /// QR code ticket data — frontend renders this as a QR image.
    Qr(String),
    /// User scanned the QR code.
    Scanned,
    /// Login successful — returns credentials for `channel.enable-plugin`.
    Done {
        zalo_session: String,
        zalo_imei: String,
        zalo_cookies: Option<String>,
    },
    /// Login failed with an error message.
    Error(String),
}

impl ZaloLoginEvent {
    /// SSE event name string.
    pub fn event_name(&self) -> &'static str {
        match self {
            Self::Qr(_) => "qr",
            Self::Scanned => "scanned",
            Self::Done { .. } => "done",
            Self::Error(_) => "error",
        }
    }

    /// Serialize the event payload as JSON.
    pub fn to_json_data(&self) -> String {
        match self {
            Self::Qr(ticket) => serde_json::to_string(&SseQrEvent {
                qrcode_data: ticket.clone(),
            })
            .unwrap_or_default(),
            Self::Scanned => "{}".into(),
            Self::Done {
                zalo_session,
                zalo_imei,
                zalo_cookies,
            } => serde_json::to_string(&SseDoneEvent {
                zalo_session: zalo_session.clone(),
                zalo_imei: zalo_imei.clone(),
                zalo_cookies: zalo_cookies.clone(),
            })
            .unwrap_or_default(),
            Self::Error(message) => serde_json::to_string(&SseErrorEvent {
                message: message.clone(),
            })
            .unwrap_or_default(),
        }
    }
}

/// Start the Zalo QR code login flow, returning a channel of SSE events.
pub fn zalo_login_stream() -> mpsc::Receiver<ZaloLoginEvent> {
    let (tx, rx) = mpsc::channel(16);
    tokio::spawn(login_flow(tx));
    rx
}

/// Internal login flow driving the real `zca-rs` QR login event sequence.
async fn login_flow(tx: mpsc::Sender<ZaloLoginEvent>) {
    let tx_clone = tx.clone();
    let callback = Box::new(move |event: LoginQRCallbackEvent| {
        let tx = tx_clone.clone();
        match event.event_type {
            LoginQRCallbackEventType::QRCodeGenerated => {
                if let LoginQRCallbackEventData::QRCodeGenerated(data) = event.data {
                    let qr_url = format!("data:image/png;base64,{}", data.image);
                    let _ = tx.try_send(ZaloLoginEvent::Qr(qr_url));
                }
            }
            LoginQRCallbackEventType::QRCodeScanned => {
                let _ = tx.try_send(ZaloLoginEvent::Scanned);
            }
            LoginQRCallbackEventType::GotLoginInfo => {
                if let LoginQRCallbackEventData::GotLoginInfo {
                    cookies,
                    imei,
                    user_agent: _,
                } = event.data
                {
                    let cookies_json = serde_json::to_string(&cookies).unwrap_or_default();
                    let _ = tx.try_send(ZaloLoginEvent::Done {
                        zalo_session: "authenticated".into(),
                        zalo_imei: imei,
                        zalo_cookies: Some(cookies_json),
                    });
                }
            }
            LoginQRCallbackEventType::QRCodeExpired => {
                let _ = tx.try_send(ZaloLoginEvent::Error("QR code expired".into()));
            }
            LoginQRCallbackEventType::QRCodeDeclined => {
                let _ = tx.try_send(ZaloLoginEvent::Error("QR login declined by user".into()));
            }
        }
    });

    let zalo_res = ZcaZalo::new(ZcaOptions::default());
    let zalo = match zalo_res {
        Ok(z) => z,
        Err(e) => {
            let _ = tx.send(ZaloLoginEvent::Error(format!("Client error: {e}"))).await;
            return;
        }
    };

    info!("Starting real Zalo SDK login_qr stream flow");
    if let Err(err) = zalo.login_qr(None, Some(callback)).await {
        error!(error = %err, "Zalo SDK login_qr flow ended with error");
        let _ = tx.send(ZaloLoginEvent::Error(format!("{err}"))).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zalo_login_event_names_and_json() {
        let qr_evt = ZaloLoginEvent::Qr("ticket_123".into());
        assert_eq!(qr_evt.event_name(), "qr");
        assert!(qr_evt.to_json_data().contains(r#""qrcodeData":"ticket_123"#));

        let scanned_evt = ZaloLoginEvent::Scanned;
        assert_eq!(scanned_evt.event_name(), "scanned");

        let done_evt = ZaloLoginEvent::Done {
            zalo_session: "sess_1".into(),
            zalo_imei: "imei_1".into(),
            zalo_cookies: None,
        };
        assert_eq!(done_evt.event_name(), "done");
        assert!(done_evt.to_json_data().contains(r#""zaloSession":"sess_1"#));

        let err_evt = ZaloLoginEvent::Error("Timeout".into());
        assert_eq!(err_evt.event_name(), "error");
        assert!(err_evt.to_json_data().contains(r#""message":"Timeout"#));
    }
}

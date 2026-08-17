use std::sync::Arc;
use tokio::sync::watch;
use tracing::{error, info, warn};
use zca_rs::models::{Message, MessageContent};

use crate::plugin::PluginCallbacks;
use crate::types::{
    MessageContentType, PluginType, UnifiedAttachment, UnifiedIncomingMessage, UnifiedMessageContent,
    UnifiedUser,
};
use super::api::ZaloApi;

/// Background listener loop for Zalo events.
pub async fn start_zalo_listener(
    api: Arc<ZaloApi>,
    callbacks: PluginCallbacks,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    info!("Zalo event listener loop started");

    let zca_opt = api.get_zca_api().await;

    if let Some(zca) = zca_opt {
        info!("Subscribing to real zca-rs WebSocket listener events");
        let mut msg_rx = zca.listener.on_message();

        let zca_bg = zca.clone();
        let listen_task = tokio::spawn(async move {
            zca_bg.listener.start(true).await;
        });

        loop {
            tokio::select! {
                res = msg_rx.recv() => {
                    match res {
                        Ok(msg) => {
                            if let Some(unified) = parse_zalo_message(msg) {
                                if let Err(e) = callbacks.message_tx.send(unified).await {
                                    error!("Failed to forward Zalo incoming message to callback channel: {e}");
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            warn!("Zalo message listener lagged by {n} messages");
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            info!("Zalo message listener broadcast channel closed");
                            break;
                        }
                    }
                }
                Ok(_) = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Zalo listener received shutdown signal");
                        zca.listener.stop();
                        break;
                    }
                }
            }
        }

        let _ = listen_task.await;
    } else {
        info!("No active zca-rs session found; running fallback tick loop");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Fallback polling loop when no active WebSocket session exists
                }
                Ok(_) = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Zalo listener received shutdown signal");
                        break;
                    }
                }
            }
        }
    }
}

/// Convert a `zca_rs::models::Message` into a `UnifiedIncomingMessage`.
pub fn parse_zalo_message(msg: Message) -> Option<UnifiedIncomingMessage> {
    match msg {
        Message::User(user_msg) => {
            if user_msg.is_self {
                return None;
            }
            let data = user_msg.data;
            let sender_id = data.uid_from.clone();
            let display_name = if data.d_name.is_empty() {
                sender_id.clone()
            } else {
                data.d_name.clone()
            };
            let (content_type, text, attachments) = extract_content(&data.content);
            let timestamp = data.ts.parse::<i64>().unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            });

            let reply_to_message_id = data.quote.as_ref().map(|q| q.global_msg_id.to_string());
            let raw = serde_json::to_value(&data).ok();

            Some(UnifiedIncomingMessage {
                owner_user_id: None,
                id: data.msg_id,
                platform: PluginType::Zalo,
                chat_id: user_msg.thread_id,
                user: UnifiedUser {
                    id: sender_id,
                    username: None,
                    display_name,
                    avatar_url: None,
                },
                content: UnifiedMessageContent {
                    content_type,
                    text,
                    attachments: if attachments.is_empty() {
                        None
                    } else {
                        Some(attachments)
                    },
                },
                timestamp,
                reply_to_message_id,
                action: None,
                raw,
            })
        }
        Message::Group(group_msg) => {
            if group_msg.is_self {
                return None;
            }
            let data = group_msg.data.base;
            let sender_id = data.uid_from.clone();
            let display_name = if data.d_name.is_empty() {
                sender_id.clone()
            } else {
                data.d_name.clone()
            };
            let (content_type, text, attachments) = extract_content(&data.content);
            let timestamp = data.ts.parse::<i64>().unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0)
            });

            let reply_to_message_id = data.quote.as_ref().map(|q| q.global_msg_id.to_string());
            let raw = serde_json::to_value(&data).ok();

            Some(UnifiedIncomingMessage {
                owner_user_id: None,
                id: data.msg_id,
                platform: PluginType::Zalo,
                chat_id: group_msg.thread_id,
                user: UnifiedUser {
                    id: sender_id,
                    username: None,
                    display_name,
                    avatar_url: None,
                },
                content: UnifiedMessageContent {
                    content_type,
                    text,
                    attachments: if attachments.is_empty() {
                        None
                    } else {
                        Some(attachments)
                    },
                },
                timestamp,
                reply_to_message_id,
                action: None,
                raw,
            })
        }
    }
}

fn extract_content(content: &MessageContent) -> (MessageContentType, String, Vec<UnifiedAttachment>) {
    match content {
        MessageContent::Text(t) => (MessageContentType::Text, t.clone(), Vec::new()),
        MessageContent::Attachment(att) => {
            let text = if !att.title.is_empty() {
                format!("{}: {}", att.title, att.href)
            } else {
                att.href.clone()
            };
            let ctype = if att.content_type.starts_with("image") {
                MessageContentType::Photo
            } else {
                MessageContentType::Document
            };
            let attachment = UnifiedAttachment {
                file_id: Some(att.href.clone()),
                file_name: if att.title.is_empty() {
                    None
                } else {
                    Some(att.title.clone())
                },
                mime_type: Some(att.content_type.clone()),
                file_size: None,
                url: Some(att.href.clone()),
            };
            (ctype, text, vec![attachment])
        }
        MessageContent::Other(map) => {
            let text = serde_json::to_string(map).unwrap_or_default();
            (MessageContentType::Text, text, Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zca_rs::models::{ParamsExt, TMessage, UserMessage};

    #[test]
    fn test_parse_user_text_message() {
        let t_msg = TMessage {
            action_id: "act_1".into(),
            msg_id: "msg_100".into(),
            cli_msg_id: "100".into(),
            msg_type: "text".into(),
            uid_from: "user_123".into(),
            id_to: "0".into(),
            d_name: "Alice".into(),
            ts: "1700000000000".into(),
            status: 0,
            content: MessageContent::Text("Hello Zalo".into()),
            notify: "".into(),
            ttl: 0,
            user_id: "user_123".into(),
            uin: "".into(),
            top_out: "".into(),
            top_out_time_out: "".into(),
            top_out_impr_time_out: "".into(),
            property_ext: None,
            params_ext: ParamsExt {
                count_unread: 0,
                contain_type: 0,
                platform_type: 0,
            },
            cmd: 0,
            st: 0,
            at: 0,
            real_msg_id: "".into(),
            quote: None,
        };

        let user_msg = UserMessage::new("bot_id", t_msg);
        let msg = Message::User(user_msg);
        let parsed = parse_zalo_message(msg).expect("Should parse message");

        assert_eq!(parsed.id, "msg_100");
        assert_eq!(parsed.platform, PluginType::Zalo);
        assert_eq!(parsed.chat_id, "user_123");
        assert_eq!(parsed.user.id, "user_123");
        assert_eq!(parsed.user.display_name, "Alice");
        assert_eq!(parsed.content.text, "Hello Zalo");
        assert_eq!(parsed.content.content_type, MessageContentType::Text);
        assert_eq!(parsed.timestamp, 1700000000000);
    }
}

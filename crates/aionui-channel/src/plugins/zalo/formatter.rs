use crate::types::{
    BotInfo, MessageContentType, OutgoingMessageType, PluginType, UnifiedIncomingMessage,
    UnifiedMessageContent, UnifiedOutgoingMessage, UnifiedUser,
};

/// Format an incoming raw text message from Zalo to `UnifiedIncomingMessage`.
pub fn format_zalo_incoming_message(
    msg_id: &str,
    user_id: &str,
    chat_id: &str,
    text_content: &str,
) -> UnifiedIncomingMessage {
    UnifiedIncomingMessage {
        owner_user_id: None,
        id: msg_id.to_string(),
        platform: PluginType::Zalo,
        chat_id: chat_id.to_string(),
        user: UnifiedUser {
            id: user_id.to_string(),
            username: None,
            display_name: format!("Zalo User {}", user_id),
            avatar_url: None,
        },
        content: UnifiedMessageContent {
            content_type: MessageContentType::Text,
            text: text_content.to_string(),
            attachments: None,
        },
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
        reply_to_message_id: None,
        action: None,
        raw: None,
    }
}

/// Convert a `UnifiedOutgoingMessage` to a Zalo-compatible text string.
pub fn format_zalo_outgoing_text(message: &UnifiedOutgoingMessage) -> String {
    match message.message_type {
        OutgoingMessageType::Text => message.text.clone().unwrap_or_default(),
        OutgoingMessageType::Image => {
            if let Some(ref url) = message.image_url {
                format!("[Image: {}] {}", url, message.text.as_deref().unwrap_or(""))
            } else {
                message.text.clone().unwrap_or_default()
            }
        }
        OutgoingMessageType::File => {
            let name = message.file_name.as_deref().unwrap_or("file");
            if let Some(ref url) = message.file_url {
                format!("[File: {} - {}] {}", name, url, message.text.as_deref().unwrap_or(""))
            } else {
                message.text.clone().unwrap_or_default()
            }
        }
        OutgoingMessageType::Buttons => message.text.clone().unwrap_or_default(),
    }
}

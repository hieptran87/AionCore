use super::formatter::*;
use crate::types::{OutgoingMessageType, PluginType, UnifiedOutgoingMessage};

#[test]
fn test_format_incoming_zalo_text_message() {
    let msg = format_zalo_incoming_message("msg_123", "user_456", "chat_789", "Hello Zalo");
    assert_eq!(msg.id, "msg_123");
    assert_eq!(msg.platform, PluginType::Zalo);
    assert_eq!(msg.chat_id, "chat_789");
    assert_eq!(msg.user.id, "user_456");
    assert_eq!(msg.content.text, "Hello Zalo");
}

#[test]
fn test_format_zalo_outgoing_text() {
    let msg = UnifiedOutgoingMessage {
        message_type: OutgoingMessageType::Text,
        text: Some("Test message".into()),
        parse_mode: None,
        buttons: None,
        keyboard: None,
        image_url: None,
        file_url: None,
        file_name: None,
        media_actions: None,
        reply_to_message_id: None,
        silent: None,
    };
    let formatted = format_zalo_outgoing_text(&msg);
    assert_eq!(formatted, "Test message");
}

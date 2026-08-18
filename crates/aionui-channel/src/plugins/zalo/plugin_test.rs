use super::plugin::ZaloPlugin;
use crate::plugin::{ChannelPlugin, PluginCallbacks};
use crate::types::{
    OutgoingMessageType, PluginConfig, PluginCredentials, PluginStatus, PluginType, UnifiedOutgoingMessage,
};

#[tokio::test]
async fn test_zalo_plugin_lifecycle() {
    let mut plugin = ZaloPlugin::new();
    assert_eq!(plugin.status(), PluginStatus::Created);
    assert_eq!(plugin.plugin_type(), PluginType::Zalo);

    let config = PluginConfig {
        credentials: PluginCredentials {
            zalo_session: Some("session_abc".into()),
            zalo_imei: Some("imei_123".into()),
            zalo_cookies: None,
            ..Default::default()
        },
        config: None,
    };

    let (tx1, _) = tokio::sync::mpsc::channel(10);
    let (tx2, _) = tokio::sync::mpsc::channel(10);
    let callbacks = PluginCallbacks {
        message_tx: tx1,
        confirm_tx: tx2,
    };

    let init_res = plugin.initialize(config, callbacks).await;
    assert!(init_res.is_ok());
    assert_eq!(plugin.status(), PluginStatus::Ready);
    assert!(plugin.bot_info().is_some());

    let start_res = plugin.start().await;
    assert!(start_res.is_ok());
    assert_eq!(plugin.status(), PluginStatus::Running);

    let msg = UnifiedOutgoingMessage {
        message_type: OutgoingMessageType::Text,
        text: Some("Hello Zalo".into()),
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

    let send_res = plugin.send_message("chat_123", msg).await;
    assert!(send_res.is_ok());
    let msg_id = send_res.unwrap();
    assert!(msg_id.starts_with("zalo_msg_"));

    let stop_res = plugin.stop().await;
    assert!(stop_res.is_ok());
    assert_eq!(plugin.status(), PluginStatus::Stopped);
}

#[tokio::test]
async fn test_zalo_plugin_send_image_message() {
    let mut plugin = ZaloPlugin::new();
    let config = PluginConfig {
        credentials: PluginCredentials {
            zalo_session: Some("session_abc".into()),
            zalo_imei: Some("imei_123".into()),
            ..Default::default()
        },
        config: None,
    };
    let (tx1, _) = tokio::sync::mpsc::channel(10);
    let (tx2, _) = tokio::sync::mpsc::channel(10);
    let callbacks = PluginCallbacks {
        message_tx: tx1,
        confirm_tx: tx2,
    };

    plugin.initialize(config, callbacks).await.unwrap();
    plugin.start().await.unwrap();

    let msg = UnifiedOutgoingMessage {
        message_type: OutgoingMessageType::Image,
        text: Some("Check out this photo".into()),
        parse_mode: None,
        buttons: None,
        keyboard: None,
        image_url: Some("https://example.com/photo.jpg".into()),
        file_url: None,
        file_name: None,
        media_actions: None,
        reply_to_message_id: None,
        silent: None,
    };

    let send_res = plugin.send_message("user_999", msg).await;
    assert!(send_res.is_ok());
    assert!(send_res.unwrap().starts_with("zalo_msg_"));
}

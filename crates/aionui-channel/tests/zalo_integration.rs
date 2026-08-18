use aionui_channel::plugins::create_plugin;
use aionui_channel::types::PluginType;

#[test]
fn test_create_zalo_plugin() {
    let plugin_opt = create_plugin(PluginType::Zalo);
    #[cfg(feature = "zalo")]
    {
        use aionui_channel::types::PluginStatus;
        assert!(plugin_opt.is_some());
        let plugin = plugin_opt.unwrap();
        assert_eq!(plugin.plugin_type(), PluginType::Zalo);
        assert_eq!(plugin.status(), PluginStatus::Created);
    }
    #[cfg(not(feature = "zalo"))]
    {
        assert!(plugin_opt.is_none());
    }
}

#[cfg(feature = "zalo")]
#[tokio::test]
async fn test_zalo_login_stream_events() {
    use aionui_channel::plugins::zalo::zalo_login_stream;

    let mut rx = zalo_login_stream();
    let event = rx.recv().await;
    assert!(event.is_some());

    let evt = event.unwrap();
    assert_eq!(evt.event_name(), "qr");
    assert!(evt.to_json_data().contains(r#""qrcodeData""#));
}

#[cfg(feature = "zalo")]
#[tokio::test]
async fn test_zalo_enable_plugin_flat_config() {
    use aionui_api_types::WebSocketMessage;
    use aionui_channel::manager::{ChannelManager, PluginFactory};
    use aionui_channel::plugins::create_plugin;
    use aionui_channel::types::PluginType;
    use aionui_db::SqliteChannelRepository;
    use aionui_db::init_database_memory;
    use aionui_realtime::EventBroadcaster;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    struct MockBroadcaster;
    impl EventBroadcaster for MockBroadcaster {
        fn broadcast(&self, _event: WebSocketMessage<serde_json::Value>) {}
    }

    let db = init_database_memory().await.unwrap();
    use aionui_db::IUserRepository;
    let user_repo = aionui_db::SqliteUserRepository::new(db.pool().clone());
    let user = user_repo.create_user("zalo_test_user", "hash").await.unwrap();

    let repo = Arc::new(SqliteChannelRepository::new(db.pool().clone()));
    let bc = Arc::new(MockBroadcaster);
    let (msg_tx, _) = mpsc::channel(16);
    let (confirm_tx, _) = mpsc::channel(16);
    let manager = ChannelManager::new(repo, bc, [0x42; 32], msg_tx, confirm_tx);

    let flat_config = serde_json::json!({
        "token": r#"[{"domain":"chat.zalo.me","name":"zpw_sek","value":"123"}]"#,
        "imei": "a77be57e-602e-4358-8a8b-32f5d0916b23-a69b52f9d7f760edf3fd052bcda2542f"
    });

    let factory: PluginFactory = Box::new(|pt: PluginType| create_plugin(pt));
    let result = manager.enable_plugin(&user.id, "zalo", &flat_config, &factory).await;
    assert!(
        result.is_ok(),
        "enable_plugin should accept flat token + imei payload: {:?}",
        result.err()
    );
}

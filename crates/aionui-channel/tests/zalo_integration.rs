use aionui_channel::plugins::create_plugin;
use aionui_channel::types::{PluginStatus, PluginType};

#[test]
fn test_create_zalo_plugin() {
    let plugin_opt = create_plugin(PluginType::Zalo);
    #[cfg(feature = "zalo")]
    {
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

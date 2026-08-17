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

use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::info;

use crate::error::ChannelError;
use crate::plugin::{ChannelPlugin, PluginCallbacks};
use crate::types::{BotInfo, OutgoingMessageType, PluginConfig, PluginStatus, PluginType, UnifiedOutgoingMessage};

use super::api::ZaloApi;
use super::formatter::format_zalo_outgoing_text;
use super::listener::start_zalo_listener;

/// Zalo platform channel plugin.
pub struct ZaloPlugin {
    status: PluginStatus,
    bot_info: Option<BotInfo>,
    last_error: Option<String>,
    callbacks: Option<PluginCallbacks>,
    api: Option<Arc<ZaloApi>>,
    listener_handle: Option<JoinHandle<()>>,
    shutdown_tx: Option<watch::Sender<bool>>,
}

impl Default for ZaloPlugin {
    fn default() -> Self {
        Self {
            status: PluginStatus::Created,
            bot_info: None,
            last_error: None,
            callbacks: None,
            api: None,
            listener_handle: None,
            shutdown_tx: None,
        }
    }
}

impl ZaloPlugin {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl ChannelPlugin for ZaloPlugin {
    async fn initialize(&mut self, config: PluginConfig, callbacks: PluginCallbacks) -> Result<(), ChannelError> {
        self.status = PluginStatus::Initializing;

        let session = config
            .credentials
            .zalo_session
            .as_deref()
            .or_else(|| config.credentials.extra.get("session").and_then(|v| v.as_str()))
            .unwrap_or_default();

        let imei = config
            .credentials
            .zalo_imei
            .as_deref()
            .or_else(|| config.credentials.extra.get("imei").and_then(|v| v.as_str()))
            .unwrap_or("default_imei");

        let cookies_raw = config
            .credentials
            .zalo_cookies
            .as_deref()
            .or(config.credentials.token.as_deref())
            .or_else(|| config.credentials.extra.get("cookies").and_then(|v| v.as_str()));

        let creds = super::types::build_zalo_credentials(session, imei, cookies_raw);
        let api = match ZaloApi::login_with_credentials(creds).await {
            Ok(real_api) => real_api,
            Err(_) => {
                let mut fallback = ZaloApi::new(session, imei);
                if let Some(cookies) = cookies_raw {
                    fallback = fallback.with_cookies(cookies);
                }
                fallback
            }
        };

        self.api = Some(Arc::new(api));
        self.bot_info = Some(BotInfo {
            id: "zalo_bot".to_string(),
            username: Some("zalo_bot".to_string()),
            display_name: "Zalo Bot".to_string(),
        });
        self.callbacks = Some(callbacks);
        self.status = PluginStatus::Ready;
        info!("ZaloPlugin initialized");
        Ok(())
    }

    async fn start(&mut self) -> Result<(), ChannelError> {
        if self.status != PluginStatus::Ready && self.status != PluginStatus::Stopped {
            return Err(ChannelError::InvalidConfig(format!(
                "Cannot start plugin in state {:?}",
                self.status
            )));
        }

        self.status = PluginStatus::Starting;
        let callbacks = self
            .callbacks
            .clone()
            .ok_or_else(|| ChannelError::InvalidConfig("Callbacks not initialized".into()))?;

        let api = self
            .api
            .clone()
            .ok_or_else(|| ChannelError::InvalidConfig("ZaloApi not initialized".into()))?;

        let (tx, rx) = watch::channel(false);
        self.shutdown_tx = Some(tx);

        let handle = tokio::spawn(async move {
            start_zalo_listener(api, callbacks, rx).await;
        });

        self.listener_handle = Some(handle);
        self.status = PluginStatus::Running;
        info!("ZaloPlugin started");
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), ChannelError> {
        if self.status != PluginStatus::Running {
            return Ok(());
        }

        self.status = PluginStatus::Stopping;
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }

        if let Some(handle) = self.listener_handle.take() {
            let _ = handle.await;
        }

        self.status = PluginStatus::Stopped;
        info!("ZaloPlugin stopped");
        Ok(())
    }

    fn status(&self) -> PluginStatus {
        self.status
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Zalo
    }

    fn bot_info(&self) -> Option<&BotInfo> {
        self.bot_info.as_ref()
    }

    fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    fn active_user_count(&self) -> usize {
        0
    }

    async fn send_message(&self, chat_id: &str, message: UnifiedOutgoingMessage) -> Result<String, ChannelError> {
        if self.status != PluginStatus::Running {
            return Err(ChannelError::PlatformApi("Zalo plugin is not running".into()));
        }

        let api = self
            .api
            .as_ref()
            .ok_or_else(|| ChannelError::PlatformApi("ZaloApi not initialized".into()))?;

        match message.message_type {
            OutgoingMessageType::Image if message.image_url.is_some() => {
                let url = message.image_url.as_deref().unwrap();
                let caption = message.text.as_deref().unwrap_or("");
                api.send_image_from_url(chat_id, url, caption)
                    .await
                    .map_err(|e| ChannelError::MessageSendFailed(format!("Zalo send image failed: {e}")))
            }
            _ => {
                let formatted_text = format_zalo_outgoing_text(&message);
                api.send_text(chat_id, &formatted_text)
                    .await
                    .map_err(|e| ChannelError::MessageSendFailed(format!("Zalo send failed: {e}")))
            }
        }
    }

    async fn edit_message(
        &self,
        chat_id: &str,
        _message_id: &str,
        message: UnifiedOutgoingMessage,
    ) -> Result<(), ChannelError> {
        let _ = self.send_message(chat_id, message).await?;
        Ok(())
    }
}

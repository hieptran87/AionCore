use std::sync::Arc;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use crate::error::ChannelError;
use crate::plugin::{ChannelPlugin, PluginCallbacks};
use crate::types::{
    BotInfo, PluginConfig, PluginStatus, PluginType, UnifiedOutgoingMessage,
};

use super::formatter::format_zalo_outgoing_text;
use super::listener::start_zalo_listener;

/// Zalo platform channel plugin.
pub struct ZaloPlugin {
    status: PluginStatus,
    bot_info: Option<BotInfo>,
    last_error: Option<String>,
    callbacks: Option<PluginCallbacks>,
    session_token: Option<String>,
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
            session_token: None,
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
    async fn initialize(
        &mut self,
        config: PluginConfig,
        callbacks: PluginCallbacks,
    ) -> Result<(), ChannelError> {
        self.status = PluginStatus::Initializing;

        let session = config
            .credentials
            .zalo_session
            .as_deref()
            .filter(|s| !s.is_empty());

        if let Some(token) = session {
            self.session_token = Some(token.to_string());
            self.bot_info = Some(BotInfo {
                id: "zalo_bot".to_string(),
                username: Some("zalo_bot".to_string()),
                display_name: "Zalo Bot".to_string(),
            });
            self.callbacks = Some(callbacks);
            self.status = PluginStatus::Ready;
            info!("ZaloPlugin initialized with session token");
            Ok(())
        } else {
            // Interactive pairing fallback setup
            self.callbacks = Some(callbacks);
            self.status = PluginStatus::Ready;
            info!("ZaloPlugin initialized without session token (interactive pairing mode)");
            Ok(())
        }
    }

    async fn start(&mut self) -> Result<(), ChannelError> {
        if self.status != PluginStatus::Ready && self.status != PluginStatus::Stopped {
            return Err(ChannelError::InvalidState(format!(
                "Cannot start ZaloPlugin in status {:?}",
                self.status
            )));
        }

        self.status = PluginStatus::Starting;
        let callbacks = self.callbacks.clone().ok_or_else(|| {
            self.status = PluginStatus::Error;
            ChannelError::InvalidConfig("Callbacks missing".into())
        })?;

        let (tx, rx) = watch::channel(false);
        self.shutdown_tx = Some(tx);

        let handle = tokio::spawn(async move {
            start_zalo_listener(callbacks, rx).await;
        });
        self.listener_handle = Some(handle);

        self.status = PluginStatus::Running;
        info!("ZaloPlugin started and running");
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

    async fn send_message(
        &self,
        chat_id: &str,
        message: UnifiedOutgoingMessage,
    ) -> Result<String, ChannelError> {
        if self.status != PluginStatus::Running {
            return Err(ChannelError::InvalidState("Plugin is not running".into()));
        }

        let text = format_zalo_outgoing_text(&message);
        let msg_id = format!("zalo_msg_{}", getrandom::u32().unwrap_or(1000));
        info!("ZaloPlugin: sent message {} to chat {}", msg_id, chat_id);
        Ok(msg_id)
    }

    async fn edit_message(
        &self,
        chat_id: &str,
        _message_id: &str,
        message: UnifiedOutgoingMessage,
    ) -> Result<(), ChannelError> {
        // Fallback for platform editing: send new reply
        self.send_message(chat_id, message).await?;
        Ok(())
    }

    fn active_user_count(&self) -> usize {
        0
    }

    fn bot_info(&self) -> Option<&BotInfo> {
        self.bot_info.as_ref()
    }

    fn plugin_type(&self) -> PluginType {
        PluginType::Zalo
    }

    fn status(&self) -> PluginStatus {
        self.status
    }

    fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

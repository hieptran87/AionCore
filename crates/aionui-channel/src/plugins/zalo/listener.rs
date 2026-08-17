use std::sync::Arc;
use tokio::sync::watch;
use tracing::info;

use crate::plugin::PluginCallbacks;
use super::api::ZaloApi;

/// Background listener loop for Zalo events.
pub async fn start_zalo_listener(
    _api: Arc<ZaloApi>,
    _callbacks: PluginCallbacks,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    info!("Zalo event listener loop started");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                // Polling / event stream tick
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

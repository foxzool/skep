use crate::helper::discovery_flow::DiscoveryKey;
use bevy_utils::HashMap;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct ConfigEntry {
    entry_id: String,
    domain: String,
    title: String,
    data: HashMap<String, Value>,
    // runtime_data: T,
    options: HashMap<String, Value>,
    unique_id: Option<String>,
    state: ConfigEntryState,
    reason: Option<String>,
    error_reason_translation_key: Option<String>,
    error_reason_translation_placeholders: Option<HashMap<String, Value>>,
    pref_disable_new_entities: bool,
    pref_disable_polling: bool,
    version: i32,
    source: String,
    minor_version: i32,
    disabled_by: Option<ConfigEntryDisabler>,
    supports_unload: Option<bool>,
    supports_remove_device: Option<bool>,
    _supports_options: Option<bool>,
    _supports_reconfigure: Option<bool>,
    // update_listeners: Vec<UpdateListenerType>,
    // _async_cancel_retry_setup: Option<Box<dyn Fn() -> Any>>,
    // _on_unload: Option<Vec<Box<dyn Fn() -> std::future::Future<Output = ()>>>>,
    // setup_lock: tokio::sync::Mutex<()>,
    // _reauth_lock: tokio::sync::Mutex<()>,
    // _reconfigure_lock: tokio::sync::Mutex<()>,
    // _tasks: std::collections::HashSet<tokio::task::JoinHandle<()>>,
    // _background_tasks: std::collections::HashSet<tokio::task::JoinHandle<()>>,
    // _integration_for_domain: Option<loader::Integration>,
    _tries: i32,
    created_at: chrono::DateTime<chrono::Utc>,
    modified_at: chrono::DateTime<chrono::Utc>,
    discovery_keys: std::collections::HashMap<String, Vec<DiscoveryKey>>,
}

impl ConfigEntry {
    // pub fn new(entity_id: Option<String>) -> Self {
    //     let entity_id = entity_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    //     Self {
    //         entry_id: entity_id,
    //     }
    // }
}

#[derive(Debug)]
enum ConfigEntryState {
    /// The config entry has been set up successfully
    Loaded { recoverable: bool },
    /// There was an error while trying to set up this config entry
    SetupError { recoverable: bool },
    /// There was an error while trying to migrate the config entry to a new version
    MigrationError { recoverable: bool },
    /// The config entry was not ready to be set up yet, but might be later
    SetupRetry { recoverable: bool },
    /// The config entry has not been loaded
    NotLoaded { recoverable: bool },
    /// An error occurred when trying to unload the entry
    FailedUnload { recoverable: bool },
    /// The config entry is setting up
    SetupInProgress { recoverable: bool },
}

impl Default for ConfigEntryState {
    fn default() -> Self {
        Self::NotLoaded { recoverable: true }
    }
}

impl ConfigEntryState {
    pub fn recoverable(&self) -> bool {
        matches!(self, Self::Loaded { .. })
    }
}

#[derive(Debug)]
enum ConfigEntryDisabler {
    User,
}

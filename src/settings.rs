use super::api::{Channel, Deck};
use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
use std::collections::HashMap;

/// HTTP part settings
#[derive(Debug, Deserialize)]
pub struct HttpSettings {
    /// IP to bind to
    pub bind: String,
    /// Port to bind to
    pub port: u16,
    /// Websocket port
    pub ws_port: u16,
    /// Webroot to throw unmatched requests at
    pub webroot: String,
    /// Send verbose events to websocket or not
    pub more_events: bool
}

/// Logic part settings
#[derive(Debug, Deserialize)]
pub struct MixingSettings {
    /// List of decks to consider for Now Playing
    pub deck_list: Vec<Deck>,
    /// Map of decks letters to channel numbers
    pub deck_channel_map: HashMap<Deck, Channel>,
    /// Default cover art image path
    pub default_cover: String,
}

/// Common settings
#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub http: HttpSettings,
    pub mixing: MixingSettings,
    pub log_level: Option<String>
}

lazy_static! {
    static ref SHARED: ServerSettings =
        ServerSettings::read_default().expect("Configuration failure");
}

impl ServerSettings {
    /// Get the active settings instance
    pub fn shared() -> &'static Self {
        &SHARED
    }

    /// Read the settings file at default location
    pub fn read_default() -> Result<Self, ConfigError> {
        Self::read("config.toml")
    }

    /// Read the settings file at a specified location
    pub fn read(fpath: &str) -> Result<Self, ConfigError> {
        debug!("Reading settings file from {}", fpath);
        let mut s = Config::new();
        s.merge(File::with_name(fpath))?;
        s.try_into()
    }
}

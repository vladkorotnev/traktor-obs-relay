use super::api::{Channel, Deck};
use config::{Config, ConfigError, File};
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct HttpSettings {
    pub bind: String,
    pub port: u16,
    pub webroot: String,
}

#[derive(Debug, Deserialize)]
pub struct MixingSettings {
    pub deck_list: Vec<Deck>,
    pub deck_channel_map: HashMap<Deck, Channel>,
}

#[derive(Debug, Deserialize)]
pub struct ServerSettings {
    pub http: HttpSettings,
    pub mixing: MixingSettings,
}

lazy_static! {
    static ref SHARED: ServerSettings =
        ServerSettings::read_default().expect("Configuration failure");
}

impl ServerSettings {
    pub fn shared() -> &'static Self {
        &SHARED
    }

    pub fn read_default() -> Result<Self, ConfigError> {
        Self::read("config.toml")
    }

    pub fn read(fpath: &str) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name(fpath))?;
        s.try_into()
    }
}

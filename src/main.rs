#[macro_use]
extern crate rouille;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
extern crate id3;
extern crate metaflac;
extern crate tokio;
extern crate tokio_tungstenite;
extern crate infer;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

mod api;
mod http_server;
mod logic;
mod settings;
mod ws_server;

use api::{channel::*, deck::*, master_clock::*, Channel, Deck};

lazy_static! {
    pub static ref DECK_STATUS: Arc<RwLock<HashMap<Deck, DeckStatus>>> =
        Arc::new(RwLock::new(HashMap::new()));
    pub static ref MASTER_CLOCK: Arc<RwLock<MasterClock>> = Arc::new(RwLock::new(MasterClock {
        ..Default::default()
    }));
    pub static ref CHANNEL_STATUS: Arc<RwLock<HashMap<Channel, ChannelStatus>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

fn main() {
    use std::str::FromStr;
    let log_level = settings::ServerSettings::shared().log_level.clone().unwrap_or(String::from("Info"));
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::from_str(&log_level).unwrap_or(log::LevelFilter::Info))
        .init()
        .unwrap();

    // preheat channel_status
    {
        let mut chan_status = CHANNEL_STATUS.write().unwrap();
        let channel_map = &settings::ServerSettings::shared().mixing.deck_channel_map;
        for (_, channel) in channel_map.iter() {
            trace!("Preheat channel matrix data {}", channel);
            chan_status.insert(*channel, ChannelStatus { is_on_air: true });
        }
        drop(chan_status);
    }

    http_server::spawn_http();
    ws_server::spawn_ws();

    loop {
        debug!("Freezing main thread for an eternity");
        std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
    }
}

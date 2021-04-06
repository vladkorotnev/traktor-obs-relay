#[macro_use]
extern crate rouille;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate id3;
extern crate metaflac;

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

mod api;
mod http_server;
mod settings;

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
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // preheat channel_status
    {
        let mut chan_status = CHANNEL_STATUS.write().unwrap();
        let channel_map = &settings::ServerSettings::shared().mixing.deck_channel_map;
        for (_, channel) in channel_map.iter() {
            chan_status.insert(*channel, ChannelStatus { is_on_air: true });
        }
        drop(chan_status);
    }

    http_server::spawn_http();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
    }
}

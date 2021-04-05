#[macro_use]
extern crate rouille;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use rouille::Response;
use std::{io, collections::HashMap, sync::{Arc, RwLock}};

mod api;
mod settings;

use api::{Deck, Channel, deck::*, channel::*, master_clock::*};

lazy_static! {
    static ref DECK_STATUS: Arc<RwLock<HashMap<Deck, DeckStatus>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref MASTER_CLOCK: Arc<RwLock<MasterClock>> = Arc::new(RwLock::new(MasterClock { ..Default::default() }));
    static ref CHANNEL_STATUS: Arc<RwLock<HashMap<Channel, ChannelStatus>>> = Arc::new(RwLock::new(HashMap::new()));
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // preheat deck_status
    {
        let mut deck_status = DECK_STATUS.write().unwrap();
        deck_status.insert(String::from("A"), DeckStatus { ..Default::default() });
        deck_status.insert(String::from("B"), DeckStatus { ..Default::default() });
        deck_status.insert(String::from("C"), DeckStatus { ..Default::default() });
        deck_status.insert(String::from("D"), DeckStatus { ..Default::default() });
        drop(deck_status);
    }

    let cfg = &settings::ServerSettings::shared().http;
    let host = &cfg.bind;
    let port = &cfg.port;
    info!("Start API at {}:{}", host, port);

    rouille::start_server(format!("{}:{}", host, port), move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    Response::text("Point traktor API or OBS here")
                },

                (POST) (/deckLoaded/{id: Deck}) => {
                    let new_status: DeckStatus = try_or_400!(rouille::input::json_input(request));
                    info!("Loaded deck {}: {:?}", id, new_status);
                    DECK_STATUS.write().expect("RwLock failed").insert(id, new_status);
                    Response::empty_204()
                },

                (POST) (/updateDeck/{id: Deck}) => {
                    let new_status: DeckStatusUpdate = try_or_400!(rouille::input::json_input(request));
                    info!("Updated deck {}: {:?}", id, new_status);
                    if let Some(deck) = DECK_STATUS.write().expect("RwLock failed").get_mut(&id) {
                        deck.update(new_status);
                    }
                    else {
                        error!("WTF is Deck {} ???", id);
                    }
                    Response::empty_204()
                },

                (POST) (/updateMasterClock) => {
                    let new_clock: MasterClock = try_or_400!(rouille::input::json_input(request));
                    info!("Update clock {:?}", new_clock);
                    *(MASTER_CLOCK.write().expect("RwLock failed")) = new_clock;
                    Response::empty_204()
                },

                (POST) (/updateChannel/{id: Channel}) => {
                    let new_status: ChannelStatus = try_or_400!(rouille::input::json_input(request));
                    info!("Update channel {}: {:?}",id, new_status);
                    CHANNEL_STATUS.write().expect("RwLock failed").insert(id, new_status);
                    Response::empty_204()
                },
                
                _ => {
                    rouille::match_assets(&request, "./assets")
                }
            )
        })
    });
}

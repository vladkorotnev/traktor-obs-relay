#[macro_use]
extern crate rouille;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
extern crate chrono;

use rouille::Response;
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
};

mod api;
mod settings;

use api::{channel::*, deck::*, master_clock::*, Channel, Deck};

lazy_static! {
    static ref DECK_STATUS: Arc<RwLock<HashMap<Deck, DeckStatus>>> =
        Arc::new(RwLock::new(HashMap::new()));
    static ref MASTER_CLOCK: Arc<RwLock<MasterClock>> = Arc::new(RwLock::new(MasterClock {
        ..Default::default()
    }));
    static ref CHANNEL_STATUS: Arc<RwLock<HashMap<Channel, ChannelStatus>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", default)]
struct NowPlayingResponse {
    pub bpm: f32,
    pub songs_on_air: Vec<DeckStatus>,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    // preheat deck_status
    {
        let mut deck_status = DECK_STATUS.write().unwrap();
        let deck_list = &settings::ServerSettings::shared().mixing.deck_list;
        for deck_name in deck_list.iter() {
            deck_status.insert(
                String::from(deck_name),
                DeckStatus {
                    ..Default::default()
                },
            );
        }
        drop(deck_status);
    }

    // preheat channel_status
    {
        let mut chan_status = CHANNEL_STATUS.write().unwrap();
        let channel_map = &settings::ServerSettings::shared().mixing.deck_channel_map;
        for (_, channel) in channel_map.iter() {
            chan_status.insert(*channel, ChannelStatus { is_on_air: true });
        }
        drop(chan_status);
    }

    let cfg = &settings::ServerSettings::shared().http;
    let host = &cfg.bind;
    let port = &cfg.port;
    let root = cfg.webroot.clone();
    info!("Start API at {}:{} in {}", host, port, root);

    rouille::start_server(format!("{}:{}", host, port), move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    Response::text("Point traktor API or OBS here")
                },

                (POST) (/deckLoaded/{id: Deck}) => {
                    info!("Loaded deck {}", id);
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

                (GET) (/nowPlaying) => {
                    let bpm = MASTER_CLOCK.read().expect("RwLock failed").bpm;
                    let cur_decks = DECK_STATUS.read().expect("RwLock failed");
                    let cur_chans = CHANNEL_STATUS.read().expect("RwLock failed");
                    let setting = &settings::ServerSettings::shared().mixing;
                    let deck_list = setting.deck_list.clone();

                    let on_air_decks = deck_list.iter().filter(|&deck| {
                        if let Some(chan) = setting.deck_channel_map.get(deck) {
                            if let Some(chan_stat) = cur_chans.get(chan) {
                                chan_stat.is_on_air
                            }
                            else {
                                false
                            }
                        }
                        else {
                            false
                        }
                    });

                    let songs_on_air: Vec<DeckStatus> = on_air_decks.map(|deck| cur_decks.get(deck) ).filter(|opt| opt.is_some()).map(|opt| opt.unwrap().clone() ).filter(|stat| stat.is_playing).collect();

                    Response::json(&NowPlayingResponse {
                        bpm,
                        songs_on_air
                    })
                },

                _ => {
                    rouille::match_assets(&request, &root)
                }
            )
        })
    });
}

use super::{
    api::{channel::*, deck::*, master_clock::*, Channel, Deck},
    settings, CHANNEL_STATUS, DECK_STATUS, MASTER_CLOCK,
};
use rouille::Response;
use std::io;

pub fn spawn_http() {
    std::thread::spawn(move || {
        debug!("Starting http server thread");
        start_http();
    });
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", default)]
struct NowPlayingResponse {
    pub bpm: f32,
    pub songs_on_air: Vec<DeckStatus>,
}

impl NowPlayingResponse {
    pub fn create() -> Self {
        let bpm = MASTER_CLOCK.read().expect("RwLock failed").bpm;
        let songs_on_air = super::logic::get_songs_on_air();
        Self { songs_on_air, bpm }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase", default)]
struct BpmResponse {
    pub bpm: f32,
    pub master_deck: Option<Deck>,
}

impl BpmResponse {
    pub fn from(clock: &MasterClock) -> Self {
        Self {
            bpm: clock.bpm,
            master_deck: clock.deck.clone(),
        }
    }
}

fn start_http() {
    let cfg = &settings::ServerSettings::shared().http;
    let host = &cfg.bind;
    let port = &cfg.port;
    let root = cfg.webroot.clone();
    info!("Start HTTP at {}:{} in {}", host, port, root);

    rouille::start_server(format!("{}:{}", host, port), move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    Response::text("Point traktor API or OBS here")
                },

                (POST) (/deckLoaded/{id: Deck}) => {
                    info!("Loaded deck {}", id);
                    let mut new_status: DeckStatus = try_or_400!(rouille::input::json_input(request));
                    new_status.deck = Some(id.clone());
                    info!("Loaded deck {} {:?}", id, new_status);
                    DECK_STATUS.write().expect("RwLock failed").insert(id, new_status);
                    super::ws_server::ws_push(&NowPlayingResponse::create());
                    Response::empty_204()
                },

                (POST) (/updateDeck/{id: Deck}) => {
                    let new_status: DeckStatusUpdate = try_or_400!(rouille::input::json_input(request));
                    info!("Updated deck {}: {:?}", id, new_status);
                    if let Some(deck) = DECK_STATUS.write().expect("RwLock failed").get_mut(&id) {
                        if deck.update(new_status) {
                            super::ws_server::ws_push(&NowPlayingResponse::create());
                        }
                    }
                    else {
                        error!("WTF is Deck {} ???", id);
                    }
                    Response::empty_204()
                },

                (POST) (/updateMasterClock) => {
                    let new_clock: MasterClock = try_or_400!(rouille::input::json_input(request));
                    info!("Update clock {:?}", new_clock);
                    super::ws_server::ws_push(&BpmResponse::from(&new_clock));
                    *(MASTER_CLOCK.write().expect("RwLock failed")) = new_clock;
                    Response::empty_204()
                },

                (POST) (/updateChannel/{id: Channel}) => {
                    let new_status: ChannelStatus = try_or_400!(rouille::input::json_input(request));
                    info!("Update channel {}: {:?}",id, new_status);
                    CHANNEL_STATUS.write().expect("RwLock failed").insert(id, new_status);
                    super::ws_server::ws_push(&NowPlayingResponse::create());
                    Response::empty_204()
                },

                (GET) (/nowPlaying) => {
                    Response::json(&NowPlayingResponse::create())
                },

                (GET) (/artwork/{deck_id: Deck}) => {
                    match super::logic::get_deck_artwork(deck_id) {
                        None => Response::empty_404(),
                        Some(art) => Response::from_data(art.mime_type, art.data)
                    }
                },

                _ => {
                    rouille::match_assets(&request, &root)
                }
            )
        })
    });
}

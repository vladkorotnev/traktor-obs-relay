use super::{
    api::{channel::*, deck::*, master_clock::*, Channel, Deck},
    settings, CHANNEL_STATUS, DECK_STATUS, MASTER_CLOCK,
};
use rouille::Response;
use std::collections::HashMap;

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
    pub ticked_deck: Option<Deck>
}

impl NowPlayingResponse {
    pub fn create(
        clock: &MasterClock,
        cur_decks: &HashMap<Deck, DeckStatus>,
        cur_chans: &HashMap<Channel, ChannelStatus>,
    ) -> Self {
        let bpm = clock.bpm;
        let songs_on_air = super::logic::get_songs_on_air(cur_decks, cur_chans);
        Self { songs_on_air, bpm, ticked_deck: None }
    }

    pub fn tick(
        clock: &MasterClock,
        cur_decks: &HashMap<Deck, DeckStatus>,
        cur_chans: &HashMap<Channel, ChannelStatus>,
        tick_reason: Deck
    ) -> Self {
        let bpm = clock.bpm;
        let songs_on_air = super::logic::get_songs_on_air(cur_decks, cur_chans);
        Self { songs_on_air, bpm, ticked_deck: Some(tick_reason) }
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

fn format_time(duration: std::time::Duration) -> String {
    let secs_part = match duration.as_secs().checked_mul(1_000_000_000) {
        Some(v) => v,
        None => return format!("{}s", duration.as_secs() as f64),
    };

    let duration_in_ns = secs_part + u64::from(duration.subsec_nanos());

    if duration_in_ns < 1_000 {
        format!("{}ns", duration_in_ns)
    } else if duration_in_ns < 1_000_000 {
        format!("{:.1}us", duration_in_ns as f64 / 1_000.0)
    } else if duration_in_ns < 1_000_000_000 {
        format!("{:.1}ms", duration_in_ns as f64 / 1_000_000.0)
    } else {
        format!("{:.1}s", duration_in_ns as f64 / 1_000_000_000.0)
    }
}

fn start_http() {
    let cfg = &settings::ServerSettings::shared();
    let host = &cfg.http.bind;
    let port = &cfg.http.port;
    let root = cfg.http.webroot.clone();
    let default_cover = cfg.mixing.default_cover.clone();
    info!("Start HTTP at {}:{} in {}", host, port, root);

    let log_ok = |req: &rouille::Request, resp: &Response, elapsed: std::time::Duration| {
        info!("{} {}: rslt={} time={}", req.method(), req.raw_url(), resp.status_code, format_time(elapsed));
    };
    let log_err = |req: &rouille::Request, _elap: std::time::Duration| {
        error!("Handler panicked: {} {}", req.method(), req.raw_url());
    };

    rouille::start_server(format!("{}:{}", host, port), move |request| {
        rouille::log_custom(&request, log_ok, log_err, || {
            router!(request,
                (GET) (/) => {
                    Response::text("Point traktor API or OBS here")
                },

                (POST) (/deckLoaded/{id: Deck}) => {
                    trace!("Deck load API call");
                    let mut new_status: DeckStatus = try_or_400!(rouille::input::json_input(request));
                    new_status.deck = Some(id.clone());
                    debug!("Loaded deck {} {:?}", id, new_status);
                    let mut decks = DECK_STATUS.write().expect("RwLock failed");
                    decks.insert(id, new_status);
                    let chans = CHANNEL_STATUS.read().expect("RwLock failed");
                    let clock = MASTER_CLOCK.read().expect("RwLock failed");
                    super::ws_server::ws_push(&NowPlayingResponse::create(&clock, &decks, &chans));
                    Response::empty_204()
                },

                (POST) (/updateDeck/{id: Deck}) => {
                    trace!("Deck update API call");
                    let new_status: DeckStatusUpdate = try_or_400!(rouille::input::json_input(request));
                    debug!("Updated deck {}: {:?}", id, new_status);
                    let mut decks = DECK_STATUS.write().expect("RwLock failed");

                    if let Some(deck) = decks.get_mut(&id) {
                        if deck.update(new_status) {
                            let chans = CHANNEL_STATUS.read().expect("RwLock failed");
                            let clock = MASTER_CLOCK.read().expect("RwLock failed");
                            super::ws_server::ws_push(&NowPlayingResponse::tick(&clock, &decks, &chans, id));
                        }
                    }
                    else {
                        error!("Deck {} is not known (yet) but update event was received!", id);
                    }
                    Response::empty_204()
                },

                (POST) (/updateMasterClock) => {
                    trace!("Clock update API call");
                    let new_clock: MasterClock = try_or_400!(rouille::input::json_input(request));
                    debug!("Update clock {:?}", new_clock);
                    super::ws_server::ws_push(&BpmResponse::from(&new_clock));
                    *(MASTER_CLOCK.write().expect("RwLock failed")) = new_clock;
                    Response::empty_204()
                },

                (POST) (/updateChannel/{id: Channel}) => {
                    trace!("Update channel API call");
                    let new_status: ChannelStatus = try_or_400!(rouille::input::json_input(request));
                    debug!("Update channel {}: {:?}",id, new_status);
                    let mut chans = CHANNEL_STATUS.write().expect("RwLock failed");
                    chans.insert(id, new_status);
                    let clock = MASTER_CLOCK.read().expect("RwLock failed");
                    let decks = DECK_STATUS.read().expect("RwLock failed");

                    super::ws_server::ws_push(&NowPlayingResponse::create(&clock, &decks, &chans));
                    Response::empty_204()
                },

                (GET) (/nowPlaying) => {
                    trace!("Now playing info API call");
                    let chans = CHANNEL_STATUS.read().expect("RwLock failed");
                    let clock = MASTER_CLOCK.read().expect("RwLock failed");
                    let decks = DECK_STATUS.read().expect("RwLock failed");
                    Response::json(&NowPlayingResponse::create(&clock, &decks, &chans))
                },

                (GET) (/artwork/{deck_id: Deck}) => {
                    trace!("Artwork get over HTTP");
                    let decks = DECK_STATUS.read().expect("RwLock failed");

                    match super::logic::get_deck_artwork(&deck_id, &decks) {
                        None => {
                            for ftype in [ ("jpg", "image/jpeg"), ("jpeg", "image/jpeg"), ("png", "image/png") ].iter() {
                                match super::logic::get_deck_assoc_file(&deck_id, &decks, ftype.0) {
                                    None => continue,
                                    Some(data) => {
                                        return Response::from_data(ftype.1, data).with_no_cache()
                                    }
                                }
                            }

                            let file_path = std::path::Path::new(&default_cover);
                            if file_path.exists() {
                                if let Ok(Some(mime)) = infer::get_from_path(file_path) {
                                    if mime.matcher_type() == infer::MatcherType::IMAGE {
                                        trace!("Sending default artwork for deck {}", deck_id);
                                        return Response::from_file(mime.mime_type(), std::fs::File::open(file_path).unwrap()).with_no_cache();
                                    }
                                    else {
                                        error!("File {} is not an image file: {}", file_path.display(), mime);
                                        return Response::empty_406().with_no_cache();
                                    }
                                }
                                else {
                                    error!("Could not find mime type of {}", file_path.display());
                                    return Response::empty_406().with_no_cache();
                                }
                            }
                            else {
                                error!("Could not find default artwork file: {}", file_path.display());
                                return Response::empty_404().with_no_cache();
                            }
                        },
                        Some(art) => {
                            trace!("Sending artwork for deck {}", deck_id);
                            Response::from_data(art.mime_type, art.data).with_no_cache()
                        }
                    }
                },

                (GET) (/subtitles/{deck_id: Deck}) => {
                    trace!("Subtitles get over HTTP");
                    let decks = DECK_STATUS.read().expect("RwLock failed");
                    match super::logic::get_deck_assoc_file(&deck_id, &decks, "ass") {
                        None => {
                            Response::empty_404().with_no_cache()
                        },
                        Some(text) => {
                            Response::from_data("text/plain", text).with_no_cache()
                        }
                    }
                },

                (GET) (/video/{deck_id: Deck}) => {
                    trace!("Video get over HTTP");
                    let decks = DECK_STATUS.read().expect("RwLock failed");
                    for ftype in [ ("mp4", "video/mp4"), ("webm", "video/webm") ].iter() {
                        match super::logic::get_deck_assoc_file(&deck_id, &decks, ftype.0) {
                            None => continue,
                            Some(data) => {
                                return Response::from_data(ftype.1, data).with_no_cache()
                            }
                        }
                    }
                    Response::empty_404().with_no_cache()
                },

                _ => {
                    rouille::match_assets(&request, &root).with_no_cache()
                }
            )
        })
    });
}

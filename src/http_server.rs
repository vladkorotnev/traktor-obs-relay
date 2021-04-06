use super::{
    api::{channel::*, deck::*, master_clock::*, Channel, Deck},
    settings, CHANNEL_STATUS, DECK_STATUS, MASTER_CLOCK,
};
use rouille::Response;
use std::{io, path::Path};

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

                    let songs_on_air: Vec<DeckStatus> = on_air_decks.map(|deck| cur_decks.get(deck) )
                                                            .filter(|opt| opt.is_some())
                                                            .map(|opt| opt.unwrap().clone() )
                                                            .filter(|stat| stat.is_playing)
                                                            .collect();

                    Response::json(&NowPlayingResponse {
                        bpm,
                        songs_on_air
                    })
                },

                (GET) (/artwork/{deck_id: Deck}) => {
                    if let Some(deck) = DECK_STATUS.read().expect("RwLock failed").get(&deck_id) {
                        let fpath = &deck.file_path;
                        info!("Get artwork of deck {}: {}", deck_id, fpath);
                        let file_path = Path::new(&fpath);
                        if !file_path.exists() {
                            Response::empty_404()
                        }
                        else {
                            if let Some(extz) = file_path.extension() {
                                match extz.to_string_lossy().to_lowercase().as_str() {
                                    "flac" => {
                                        if let Ok(tags) = metaflac::Tag::read_from_path(file_path) {
                                            if let Some(pic) = tags.pictures().nth(0) {
                                                return Response::from_data(pic.mime_type.clone(), pic.data.clone());
                                            }
                                        }

                                        Response::empty_400()
                                    },
                                    "mp3" => {
                                        if let Ok(tags) = id3::Tag::read_from_path(file_path) {
                                            if let Some(pic) = tags.pictures().nth(0) {
                                                return Response::from_data(pic.mime_type.clone(), pic.data.clone());
                                            }
                                        }

                                        Response::empty_400()
                                    },
                                    _ => Response::empty_406()
                                }
                            }
                            else {
                                Response::empty_406()
                            }
                        }
                    }
                    else {
                        error!("WTF is Deck {} ???", deck_id);
                        Response::empty_404()
                    }
                },

                _ => {
                    rouille::match_assets(&request, &root)
                }
            )
        })
    });
}

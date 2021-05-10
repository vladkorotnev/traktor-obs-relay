use super::{
    api::{channel::*, deck::*, Channel, Deck},
    settings,
};
use std::collections::HashMap;
use std::path::Path;
use std::fs::read_to_string;

pub fn get_songs_on_air(
    cur_decks: &HashMap<Deck, DeckStatus>,
    cur_chans: &HashMap<Channel, ChannelStatus>,
) -> Vec<DeckStatus> {
    trace!("Get songs currently on air");
    let setting = &settings::ServerSettings::shared().mixing;
    let deck_list = setting.deck_list.clone();

    let on_air_decks = deck_list.iter().filter(|&deck| {
        if let Some(chan) = setting.deck_channel_map.get(deck) {
            if let Some(chan_stat) = cur_chans.get(chan) {
                chan_stat.is_on_air
            } else {
                false
            }
        } else {
            false
        }
    });
    debug!("Decks on air: {:?}", on_air_decks);

    let songs_on_air: Vec<DeckStatus> = on_air_decks
        .map(|deck| cur_decks.get(deck))
        .filter(|opt| opt.is_some())
        .map(|opt| opt.unwrap().clone())
        .filter(|stat| stat.is_playing)
        .collect();

    songs_on_air
}

pub struct Artwork {
    pub mime_type: String,
    pub data: Vec<u8>,
}

pub fn get_deck_artwork(deck_id: &Deck, decks: &HashMap<Deck, DeckStatus>) -> Option<Artwork> {
    if let Some(deck) = decks.get(deck_id) {
        let fpath = &deck.file_path;
        trace!("Get artwork of deck {}: {}", deck_id, fpath);
        let file_path = Path::new(&fpath);
        if !file_path.exists() {
            error!("Deck {} is playing a nonexistent file {}", deck_id, file_path.display());
            None
        } else {
            if let Some(extz) = file_path.extension() {
                match extz.to_string_lossy().to_lowercase().as_str() {
                    "flac" => {
                        if let Ok(tags) = metaflac::Tag::read_from_path(file_path) {
                            if let Some(pic) = tags.pictures().nth(0) {
                                return Some(Artwork {
                                    mime_type: pic.mime_type.clone(),
                                    data: pic.data.clone(),
                                });
                            } else {
                                error!("Could not find or read picture in FLAC file: {}", file_path.display());
                            }
                        } else {
                            error!("Could not read metadata in FLAC file: {}", file_path.display());
                        }

                        None
                    }
                    "mp3" => {
                        if let Ok(tags) = id3::Tag::read_from_path(file_path) {
                            if let Some(pic) = tags.pictures().nth(0) {
                                return Some(Artwork {
                                    mime_type: pic.mime_type.clone(),
                                    data: pic.data.clone(),
                                });
                            } else {
                                error!("Could not find or read picture in MP3 file: {}", file_path.display());
                            }
                        } else {
                            error!("Could not read metadata in MP3 file: {}", file_path.display());
                        }

                        None
                    }
                    _ => {
                        error!("Unsupported file extension to extract artwork from: {}", extz.to_string_lossy());
                        None
                    },
                }
            } else {
                error!("Could not determine extension of file {}", file_path.display());
                None
            }
        }
    } else {
        error!("Could not get deck {}", deck_id);
        None
    }
}

pub fn get_deck_subtitles(deck_id: &Deck, decks: &HashMap<Deck, DeckStatus>) -> Option<String> {
    if let Some(deck) = decks.get(deck_id) {
        let fpath = &deck.file_path;
        trace!("Get artwork of deck {}: {}", deck_id, fpath);
        let file_path = Path::new(&fpath);
        if !file_path.exists() {
            error!("Deck {} is playing a nonexistent file {}", deck_id, file_path.display());
            None
        } else {
            let subtitle_path = file_path.with_extension("ass");
            if subtitle_path.exists() {
                if let Ok(content) = read_to_string(&subtitle_path) {
                    Some(content)
                } else {
                    error!("Could not read subtitles from {:?}", subtitle_path);
                    None
                }
            } else {
                trace!("Not found subtitles at path {:?}", subtitle_path);
                None
            }
        }
    } else {
        error!("Could not get deck {}", deck_id);
        None
    }
}

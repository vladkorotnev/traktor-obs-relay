use super::{
    api::{deck::*, Deck},
    settings, CHANNEL_STATUS, DECK_STATUS,
};
use std::path::Path;

pub fn get_songs_on_air() -> Vec<DeckStatus> {
    let cur_decks = DECK_STATUS.read().expect("RwLock failed");
    let cur_chans = CHANNEL_STATUS.read().expect("RwLock failed");
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

pub fn get_deck_artwork(deck_id: Deck) -> Option<Artwork> {
    if let Some(deck) = DECK_STATUS.read().expect("RwLock failed").get(&deck_id) {
        let fpath = &deck.file_path;
        info!("Get artwork of deck {}: {}", deck_id, fpath);
        let file_path = Path::new(&fpath);
        if !file_path.exists() {
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
                            }
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
                            }
                        }

                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    } else {
        None
    }
}

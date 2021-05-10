/// Describes a deck status
#[derive(Deserialize, Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeckStatus {
    /// Currently loaded audio file path
    pub file_path: String,
    /// Track title
    pub title: String,
    /// Track artist
    pub artist: String,
    /// Track album
    pub album: String,
    /// Track genre
    pub genre: String,
    /// Track comment
    pub comment: String,
    /// ???
    pub comment2: String,
    /// Track releaser label
    pub label: String,
    /// Mix type of track
    pub mix: String,
    /// Remixer name
    pub remixer: String,
    /// Key in traktor format, such as "9m" or "4d"
    pub key: String,
    /// User-readable key name
    pub key_text: String,
    /// Grid offset from start of track in seconds
    pub grid_offset: f32,
    /// Length of track in seconds
    pub track_length: f32,
    /// Time elapsed into the track in seconds
    pub elapsed_time: f32,
    /// Next cue position in seconds
    #[serde(default)]
    pub next_cue_pos: Option<f32>,
    /// BPM
    pub bpm: f32,
    /// Tempo
    pub tempo: f32,
    /// Key of track after the key adjustment on the mixer is applied
    pub resulting_key: String,
    /// Whether the track is playing
    pub is_playing: bool,
    /// Whether the track is synced
    pub is_synced: bool,
    /// Whether the track is keylocked
    pub is_key_lock_on: bool,
    /// Deck letter the track is playing on
    #[serde(default)]
    pub deck: Option<String>,
}

impl DeckStatus {
    /// Update the status entry from a delta object, returns whether the change affects the Now Playing status
    pub fn update(&mut self, delta: DeckStatusUpdate) -> bool {
        use crate::settings::ServerSettings;

        trace!("Updating deck {:?} with delta: {:?}", self.deck, delta);
        let mut rslt = false;
        if let Some(time) = delta.elapsed_time {
            self.elapsed_time = time;
            rslt |= ServerSettings::shared().http.more_events;
        }
        if let Some(playing) = delta.is_playing {
            self.is_playing = playing;
            rslt |= true;
        }
        if let Some(sync) = delta.is_synced {
            self.is_synced = sync;
        }
        if let Some(key) = delta.is_key_lock_on {
            self.is_key_lock_on = key;
        }
        if let Some(tempo) = delta.tempo {
            self.tempo = tempo;
            rslt |= ServerSettings::shared().http.more_events;
        }
        if let Some(res_key) = delta.resulting_key {
            self.resulting_key = res_key;
        }
        rslt
    }
}

/// Delta object for DeckStatus
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatusUpdate {
    /// Time elapsed into the track in seconds
    #[serde(default)]
    pub elapsed_time: Option<f32>,
    /// Next cue position in seconds
    #[serde(default)]
    pub next_cue_pos: Option<f32>,
    /// Whether the track is playing
    #[serde(default)]
    pub is_playing: Option<bool>,
    /// Whether the track is synced
    #[serde(default)]
    pub is_synced: Option<bool>,
    /// Whether the track is keylocked
    #[serde(default)]
    pub is_key_lock_on: Option<bool>,
    /// Tempo
    #[serde(default)]
    pub tempo: Option<f32>,
    /// Key of track after the key adjustment on the mixer is applied
    #[serde(default)]
    pub resulting_key: Option<String>,
}

#[derive(Deserialize, Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeckStatus {
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub comment: String,
    pub comment2: String,
    pub label: String,
    pub mix: String,
    pub remixer: String,
    pub key: String,
    pub key_text: String,
    pub grid_offset: f32,
    pub track_length: f32,
    pub elapsed_time: f32,
    #[serde(default)]
    pub next_cue_pos: Option<f32>,
    pub bpm: f32,
    pub tempo: f32,
    pub resulting_key: String,
    pub is_playing: bool,
    pub is_synced: bool,
    pub is_key_lock_on: bool,
    #[serde(default)]
    pub deck: Option<String>,
}

impl DeckStatus {
    pub fn update(&mut self, delta: DeckStatusUpdate) {
        if let Some(time) = delta.elapsed_time {
            self.elapsed_time = time;
        }
        if let Some(playing) = delta.is_playing {
            self.is_playing = playing;
        }
        if let Some(sync) = delta.is_synced {
            self.is_synced = sync;
        }
        if let Some(key) = delta.is_key_lock_on {
            self.is_key_lock_on = key;
        }
        if let Some(tempo) = delta.tempo {
            self.tempo = tempo;
        }
        if let Some(res_key) = delta.resulting_key {
            self.resulting_key = res_key;
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatusUpdate {
    #[serde(default)]
    pub elapsed_time: Option<f32>,
    #[serde(default)]
    pub next_cue_pos: Option<f32>,
    #[serde(default)]
    pub is_playing: Option<bool>,
    #[serde(default)]
    pub is_synced: Option<bool>,
    #[serde(default)]
    pub is_key_lock_on: Option<bool>,
    #[serde(default)]
    pub tempo: Option<f32>,
    #[serde(default)]
    pub resulting_key: Option<String>,
}

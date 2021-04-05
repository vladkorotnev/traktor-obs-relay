
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatus {
    pub file_path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub comment: Option<String>,
    pub comment2: Option<String>,
    pub label: Option<String>,
    pub mix: Option<String>,
    pub remixer: Option<String>,
    pub key: i32,
    pub key_text: String,
    pub grid_offset: i32,
    pub track_length: i32,
    pub elapsed_time: i32,
    pub next_cue_pos: Option<i32>,
    pub bpm: u32,
    pub tempo: u32,
    pub resulting_key: u32,
    pub is_playing: bool,
    pub is_synced: bool,
    pub is_key_lock_on: bool
}

impl DeckStatus {
    pub fn update(&mut self, delta: DeckStatusUpdate) {
        if let Some(time) = delta.elapsed_time { self.elapsed_time = time; }
        if let Some(playing) = delta.is_playing { self.is_playing = playing; }
        if let Some(sync) = delta.is_synced { self.is_synced = sync; }
        if let Some(key) = delta.is_key_lock_on { self.is_key_lock_on = key; }
        if let Some(tempo) = delta.tempo { self.tempo = tempo; }
        if let Some(res_key) = delta.resulting_key { self.resulting_key = res_key; }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeckStatusUpdate {
    pub elapsed_time: Option<i32>,
    pub next_cue_pos: Option<i32>,
    pub is_playing: Option<bool>,
    pub is_synced: Option<bool>,
    pub is_key_lock_on: Option<bool>,
    pub tempo: Option<u32>,
    pub resulting_key: Option<u32>
}
use super::*;

/// Master clock status
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MasterClock {
    /// Current master deck if any
    pub deck: Option<Deck>,
    /// Current BPM
    pub bpm: f32,
}

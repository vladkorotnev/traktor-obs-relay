/// Describes an audio channel status
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatus {
    /// Whether the channel is on air (hearable by listeners)
    pub is_on_air: bool,
}

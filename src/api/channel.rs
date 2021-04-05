
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelStatus {
    pub is_on_air: bool,
}
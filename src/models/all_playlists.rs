use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyAllPlaylistsRes {
    pub href: String,
    pub limit: i64,
    pub next: Value,
    pub offset: i64,
    pub previous: Value,
    pub total: i64,
    pub items: Items,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Items {
    pub collaborative: bool,
    pub description: String,
    #[serde(rename = "external_urls")]
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub owner: Owner,
    pub public: bool,
    #[serde(rename = "snapshot_id")]
    pub snapshot_id: String,
    pub tracks: Tracks,
    #[serde(rename = "type")]
    pub type_field: String,
    pub uri: String,
    #[serde(rename = "primary_color")]
    pub primary_color: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub url: String,
    pub height: Value,
    pub width: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    #[serde(rename = "external_urls")]
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub uri: String,
    #[serde(rename = "display_name")]
    pub display_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracks {
    pub href: String,
    pub total: i64,
}

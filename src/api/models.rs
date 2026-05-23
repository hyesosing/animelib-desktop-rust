// src/api/models.rs
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AnimeCatalogResponse {
    pub data: Vec<AnimeItem>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnimeItem {
    pub id: u64,
    #[serde(rename = "slug_url")]
    pub slug: String,
    pub name: String,
    pub rus_name: Option<String>,
    pub eng_name: Option<String>,
    pub cover: Option<CoverImage>,
    #[serde(alias = "ageRestriction")]
    pub age_restriction: Option<AgeRestriction>,
    #[serde(rename = "type")]
    pub item_type: Option<ItemType>,
    pub status: Option<TitleStatus>,
    pub episodes_count: Option<u32>,
    pub rating: Option<RatingInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ItemType {
    pub id: u32,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RatingInfo {
    pub average: Option<String>,
    #[serde(rename = "averageFormated")]
    pub average_formatted: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CoverImage {
    pub thumbnail: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AgeRestriction {
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TitleStatus {
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PaginationMeta {
    pub current_page: u32,
    pub last_page: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnimeDetailResponse {
    pub data: AnimeDetail,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnimeDetail {
    pub id: u64,
    #[serde(rename = "slug_url")]
    pub slug: String,
    pub name: String,
    pub rus_name: Option<String>,
    pub cover: Option<CoverImage>,
    pub summary: Option<serde_json::Value>,
    pub genres: Option<Vec<Genre>>,
    pub episodes_count: Option<u32>,
    pub rating: Option<RatingInfo>,
    pub status: Option<TitleStatus>,
    pub episodes: Option<Vec<EpisodeItem>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Genre {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EpisodesResponse {
    pub data: Vec<EpisodeItem>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EpisodeItem {
    pub id: u64,
    pub number: String,
    pub name: Option<String>,
    #[serde(default)]
    pub players: Vec<Player>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub team: Option<TeamInfo>,
    pub src: Option<String>,
    pub player: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TeamInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HomeDataResponse {
    pub data: HomeData,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HomeData {
    pub latest_updates: Option<Vec<AnimeItem>>,
    pub popular: Option<Vec<AnimeItem>>,
    pub slider: Option<Vec<AnimeItem>>,
    pub newest: Option<Vec<AnimeItem>>,
}

// src/api/endpoints.rs
pub const CATALOG: &str = "/anime";
pub const ANIME_DETAIL: &str = "/anime/{slug}";
pub const EPISODES: &str = "/anime/{slug}/episodes";

pub fn build_url(base: &str, path: &str, params: &[(&str, &str)]) -> String {
    let mut url = format!("{}{}", base, path);
    if !params.is_empty() {
        let query_string: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
        url = format!("{}?{}", url, query_string.join("&"));
    }
    url
}

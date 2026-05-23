// src/api/client.rs
use super::{endpoints, models::*};
use reqwest::{header, Client};

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    #[allow(dead_code)]
    site_id: u32,
    client: Client,
}

impl ApiClient {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
        );
        headers.insert(
            header::REFERER,
            header::HeaderValue::from_static("https://v5.animelib.org/"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "Site-Id",
            header::HeaderValue::from_static("5"),
        );

        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: "https://api.cdnlibs.org/api".to_string(),
            site_id: 5,
            client,
        }
    }

    pub async fn get_catalog(
        &self,
        page: u32,
        query: Option<&str>,
    ) -> anyhow::Result<AnimeCatalogResponse> {
        let page_str = page.to_string();
        let mut params = vec![("page", page_str.as_str()), ("site_id[]", "5")];
        if let Some(q) = query {
            if !q.is_empty() {
                params.push(("q", q));
            }
        }

        let url = endpoints::build_url(&self.base_url, endpoints::CATALOG, &params);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("HTTP Error: {}", resp.status());
        }

        let text = resp.text().await?;
        let data = match serde_json::from_str::<AnimeCatalogResponse>(&text) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Catalog Parse Error: {}", e);
                std::fs::write("error.json", &text).ok();
                anyhow::bail!("Parse error: {}", e);
            }
        };
        Ok(data)
    }

    pub async fn get_anime(&self, slug: &str) -> anyhow::Result<AnimeDetail> {
        let path = endpoints::ANIME_DETAIL.replace("{slug}", slug);
        let fields = vec![
            "summary", "releaseDate", "type_id", "caution", "views", "rate_avg", "rate",
            "genres", "tags", "teams", "franchise", "authors", "publisher", "userRating",
            "moderated", "anime_status_id", "time", "episodes", "episodes_count",
            "episodesSchedule", "shiki_rate", "eng_name", "otherNames",
        ];
        let mut params = Vec::new();
        for f in fields {
            params.push(("fields[]", f));
        }
        let url = endpoints::build_url(&self.base_url, &path, &params);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("HTTP Error: {}", resp.status());
        }

        let text = resp.text().await?;
        let mut detail = match serde_json::from_str::<AnimeDetailResponse>(&text) {
            Ok(d) => d.data,
            Err(e) => {
                log::error!("Anime Detail Parse Error: {}", e);
                std::fs::write("error_detail.json", &text).ok();
                anyhow::bail!("Parse error: {}", e);
            }
        };

        // Fetch episodes
        let eps_url = format!("{}/episodes?anime_id={}", self.base_url, detail.id);
        if let Ok(resp) = self.client.get(&eps_url).send().await {
            if let Ok(eps_text) = resp.text().await {
                if let Ok(eps_data) = serde_json::from_str::<crate::api::models::EpisodesResponse>(&eps_text) {
                    detail.episodes = Some(eps_data.data);
                }
            }
        }

        Ok(detail)
    }

    pub async fn get_episode_players(&self, ep_id: u64) -> anyhow::Result<Vec<Player>> {
        let url = format!("{}/episodes/{}", self.base_url, ep_id);
        let resp = self.client.get(&url).send().await?;
        let text = resp.text().await?;
        let data = serde_json::from_str::<serde_json::Value>(&text)?;
        if let Some(players) = data.get("data").and_then(|d| d.get("players")) {
            let res: Vec<Player> = serde_json::from_value(players.clone())?;
            return Ok(res);
        }
        Ok(Vec::new())
    }

    pub async fn fetch_image_bytes(&self, url: &str) -> anyhow::Result<Vec<u8>> {
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("HTTP Error fetching image: {}", resp.status());
        }
        let bytes = resp.bytes().await?;
        Ok(bytes.to_vec())
    }

    pub async fn extract_kodik_link(&self, iframe_url: &str) -> anyhow::Result<String> {
        let url_no_proto = iframe_url.replace("https://", "").replace("http://", "").replace("//", "");
        let parts: Vec<&str> = url_no_proto.split('/').collect();
        if parts.len() < 4 {
            anyhow::bail!("Invalid Kodik URL format");
        }
        
        let k_type = parts[1];
        let k_id = parts[2];
        let k_hash = parts[3];

        let params = [
            ("type", k_type),
            ("id", k_id),
            ("hash", k_hash),
            ("uid", ""),
        ];

        let resp = self.client.post("https://kodikplayer.com/ftor")
            .form(&params)
            .send()
            .await?;
            
        let text = resp.text().await?;
        let js: serde_json::Value = serde_json::from_str(&text)?;
        
        let links_obj = js.get("links").and_then(|l| l.as_object()).ok_or_else(|| anyhow::anyhow!("No links in ftor response"))?;
        
        let mut best_src = String::new();
        for (_, streams) in links_obj {
            if let Some(stream_arr) = streams.as_array() {
                if let Some(first) = stream_arr.first() {
                    if let Some(src) = first.get("src").and_then(|s| s.as_str()) {
                        best_src = src.to_string();
                    }
                }
            }
        }
        
        if best_src.is_empty() {
            anyhow::bail!("Failed to find src in ftor response");
        }
        
        // Decode ROT18
        let mut decoded = String::new();
        for c in best_src.chars() {
            if c.is_ascii_uppercase() {
                decoded.push((((c as u8 - b'A' + 18) % 26) + b'A') as char);
            } else if c.is_ascii_lowercase() {
                decoded.push((((c as u8 - b'a' + 18) % 26) + b'a') as char);
            } else {
                decoded.push(c);
            }
        }
        
        // Base64 decode, padding manually if required, standard base64 engine ignores padding
        use base64::{Engine as _, engine::general_purpose};
        let b64_bytes = general_purpose::STANDARD.decode(format!("{}=====", decoded).trim_end_matches('='))
            .or_else(|_| general_purpose::STANDARD_NO_PAD.decode(decoded.trim_end_matches('=')))?;
        let b64_str = String::from_utf8(b64_bytes)?;
        
        let final_url = b64_str.split(":hls:").next().unwrap_or(&b64_str);
        let final_url = if final_url.starts_with("//") {
            format!("https:{}", final_url)
        } else {
            final_url.to_string()
        };
        
        Ok(final_url)
    }
}

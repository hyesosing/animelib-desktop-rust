use reqwest::blocking::Client;
use reqwest::header;

fn main() {
    let mut headers = header::HeaderMap::new();
    headers.insert("Site-Id", header::HeaderValue::from_static("5"));
    headers.insert(header::REFERER, header::HeaderValue::from_static("https://v5.animelib.org/"));
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"));
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
    
    let client = Client::builder().default_headers(headers).build().unwrap();
    let resp = client.get("https://api.cdnlibs.org/api/anime?site_id[]=5&page=1").send().unwrap();
    let text = resp.text().unwrap();
    
    std::fs::write("response.json", &text).unwrap();
    println!("Saved to response.json");
}

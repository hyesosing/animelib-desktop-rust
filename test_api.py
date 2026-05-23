import requests
import json

headers = {
    "Site-Id": "5",
    "Referer": "https://v5.animelib.org/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0",
    "Accept": "application/json"
}

resp = requests.get("https://api.cdnlibs.org/api/anime?site_id[]=5&page=1", headers=headers)
with open("response.json", "w", encoding="utf-8") as f:
    json.dump(resp.json(), f, ensure_ascii=False, indent=2)

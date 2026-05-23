# Примеры запросов к API LibSocial (Неофициальные)

В этом файле собраны готовые примеры запросов (cURL и Python `requests`) для всех основных эндпоинтов сайтов MangaLib, RanobeLib и AnimeLib.

Перед использованием убедитесь, что вы ознакомились с базовыми правилами в [README.md](README.md) (обязательные заголовки, `Site-Id` и ограничения).

---

## 📚 Манга и Ранобе

В примерах ниже используется `Site-Id: 3` (RanobeLib). Для MangaLib просто замените `Site-Id` на `1` и `Referer` на `https://mangalib.me/`.

### 1. Информация о произведении

**cURL:**
```bash
curl -X GET "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths?fields[]=summary&fields[]=genres&fields[]=authors&fields[]=chap_count" \
  -H "Site-Id: 3" \
  -H "Referer: https://ranobelib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

**Python (requests):**
```python
import requests

url = "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths"
params = {
    "fields[]": ["summary", "genres", "authors", "chap_count"]
}
headers = {
    "Site-Id": "3",
    "Referer": "https://ranobelib.me/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"
}

response = requests.get(url, headers=headers, params=params)
print(response.json())
```

### 2. Список глав

**cURL:**
```bash
curl -X GET "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths/chapters" \
  -H "Site-Id: 3" \
  -H "Referer: https://ranobelib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

**Python:**
```python
import requests

url = "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths/chapters"
headers = {
    "Site-Id": "3",
    "Referer": "https://ranobelib.me/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"
}

response = requests.get(url, headers=headers)
chapters = response.json().get("data", [])
for ch in chapters:
    print(f"Том {ch['volume']}, Глава {ch['number']}")
```

### 3. Контент главы
*(На примере 1-го тома, 1-й главы)*

**cURL:**
```bash
curl -X GET "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths/chapter?number=1&volume=1" \
  -H "Site-Id: 3" \
  -H "Referer: https://ranobelib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

**Python:**
```python
import requests

url = "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths/chapter"
params = {
    "number": 1,
    "volume": 1
    # "branch_id": 123  # Если есть несколько переводов
}
headers = {
    "Site-Id": "3",
    "Referer": "https://ranobelib.me/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"
}

response = requests.get(url, headers=headers, params=params)
data = response.json().get("data", {})
# Для Ранобе (Site-Id: 3) контент лежит в data["content"] как HTML-строка
# Для Манги (Site-Id: 1) контент лежит в data["pages"] как массив изображений
```

### 4. Поиск

**cURL (Ищем "Solo Leveling" на MangaLib):**
```bash
curl -X GET "https://api.cdnlibs.org/api/manga?q=Solo%20Leveling&site_id[]=1" \
  -H "Site-Id: 1" \
  -H "Referer: https://mangalib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

**Python:**
```python
import requests

url = "https://api.cdnlibs.org/api/manga"
params = {
    "q": "Solo Leveling",
    "site_id[]": 1
}
headers = {
    "Site-Id": "1",
    "Referer": "https://mangalib.me/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"
}

response = requests.get(url, headers=headers, params=params)
print(response.json())
```

### 5. Каталог произведений с фильтрацией

**cURL (Сортировка по высокому рейтингу):**
```bash
curl -X GET "https://api.cdnlibs.org/api/manga?site_id[]=1&sort_by=rate_avg&rate_min=50&sort_type=desc&page=1" \
  -H "Site-Id: 1" \
  -H "Referer: https://mangalib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

### 6. Топ просмотров (за неделю)

**cURL:**
```bash
curl -X GET "https://api.cdnlibs.org/api/media/top-views?page=1&popularity=3&time=week" \
  -H "Site-Id: 1" \
  -H "Referer: https://mangalib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

---

## 🎬 Аниме (AnimeLib)

Для аниме используется другой путь (`/api/anime`), `Site-Id: 5` и `Referer: https://v5.animelib.org/`.

### 7. Информация об аниме

**cURL:**
```bash
curl -X GET "https://api.cdnlibs.org/api/anime/19783--jujutsu-kaisen-2nd-season-anime?fields[]=summary&fields[]=episodes_count&fields[]=time&fields[]=shiki_rate" \
  -H "Site-Id: 5" \
  -H "Referer: https://v5.animelib.org/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

**Python:**
```python
import requests

url = "https://api.cdnlibs.org/api/anime/19783--jujutsu-kaisen-2nd-season-anime"
params = {
    "fields[]": ["summary", "episodes_count", "time", "shiki_rate"]
}
headers = {
    "Site-Id": "5",
    "Referer": "https://v5.animelib.org/",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0"
}

response = requests.get(url, headers=headers, params=params)
print(response.json())
```



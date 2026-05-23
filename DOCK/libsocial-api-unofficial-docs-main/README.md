# LibSocial API Documentation (HTTP)

> ⚠️ **Это НЕОФИЦИАЛЬНАЯ документация.** Составлена на основе реверс-инжиниринга API в **марте 2026 года**. Авторы сайтов не публиковали официальную документацию. Эндпоинты и структуры ответов могут измениться в любой момент без предупреждения.

Данный документ описывает **HTTP API** сайтов MangaLib, RanobeLib и AnimeLib. Информация не привязана к конкретному языку программирования — достаточно любого HTTP-клиента.

👉 **Полные примеры запросов на cURL и Python ищите в файле: [API_EXAMPLES.md](API_EXAMPLES.md)**

---

## Общая архитектура

Все сайты семейства lib.social работают через **единый бэкенд**. Разделение между сайтами происходит через заголовок `Site-Id`.

### Base URL

```http
https://api.cdnlibs.org
```

Зеркало (если основной недоступен):

```http
https://api.lib.social
```

### Обязательные заголовки

Каждый запрос **должен** содержать:

| Заголовок | Значение | Описание |
|-----------|----------|----------|
| `Site-Id` | `1`, `3` или `5` | Идентификатор сайта (см. таблицу ниже) |
| `Referer` | `https://{domain}/` | Домен сайта |
| `User-Agent` | Любой браузерный | Без него API может вернуть 403 |
| `Accept` | `application/json` | Формат ответа |

### Site-Id

| Site-Id | Сайт | Домен | Тип контента |
|---------|------|-------|--------------|
| `1` | MangaLib | `mangalib.me` | Манга (изображения) |
| `3` | RanobeLib | `ranobelib.me` | Ранобе / Новеллы (текст) |
| `5` | AnimeLib | `v5.animelib.org` | Аниме (видео) |

### Пример запроса (cURL)

```bash
curl -X GET "https://api.cdnlibs.org/api/manga/67008--avatar-the-last-airbender-cold-paths/chapters" \
  -H "Site-Id: 3" \
  -H "Referer: https://ranobelib.me/" \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/134.0.0.0" \
  -H "Accept: application/json"
```

---

## Эндпоинты — Манга и Ранобе

Манга и ранобе используют одни и те же эндпоинты (`/api/manga/...`). Отличие только в `Site-Id` (`1` для манги, `3` для ранобе) и формате контента глав.

### Информация о произведении

```http
GET /api/manga/{slug}?fields[]=summary&fields[]=genres&fields[]=authors&...
```

**Параметры:**

- `{slug}` — идентификатор вида `67008--avatar-the-last-airbender-cold-paths`

**Доступные fields[]:**

```	ext
background, eng_name, otherNames, summary, releaseDate, type_id, caution,
views, close_view, rate_avg, rate, genres, tags, teams, franchise, authors,
publisher, userRating, moderated, metadata, metadata.count,
metadata.close_comments, manga_status_id, chap_count, status_id, artists, format
```

**Ответ:**

```json
{
  "data": {
    "id": 67008,
    "name": "Avatar The Last Airbender: Cold Paths (Novel)",
    "rus_name": "Аватар Последний Маг Воздуха: Путь Холода",
    "eng_name": "Avatar The Last Airbender: Cold Paths",
    "slug": "avatar-the-last-airbender-cold-paths",
    "slug_url": "67008--avatar-the-last-airbender-cold-paths",
    "cover": {
      "default": "https://cover.imgslib.link/.../cover_250x350.jpg",
      "thumbnail": "..."
    },
    "type": { "id": 5, "label": "Новелла" },
    "status": { "id": 1, "label": "Завершён" },
    "summary": "Описание произведения...",
    "releaseDate": "2023",
    "views": { "total": 12345, "short": "12K" },
    "rate_avg": "8.50",
    "genres": [
      { "id": 1, "name": "Боевик" },
      { "id": 2, "name": "Фэнтези" }
    ],
    "tags": [ { "id": 10, "name": "ГГ мужчина" } ],
    "authors": [ { "id": 5, "name": "Author Name" } ],
    "teams": [ { "id": 100, "name": "Team Name" } ],
    "otherNames": ["Alternative Title"],
    "chap_count": 110,
    "format": [ { "id": 1, "name": "Веб" } ]
  }
}
```

---

### Список глав

```http
GET /api/manga/{slug}/chapters
```

**Ответ:**

```json
{
  "data": [
    {
      "id": 123456,
      "volume": "1",
      "number": "1",
      "name": "",
      "branches": [
        {
          "branch_id": null,
          "teams": [
            { "id": 100, "name": "Жнецы Рая" }
          ],
          "created_at": "2024-01-15T12:00:00.000000Z"
        }
      ]
    },
    {
      "volume": "1",
      "number": "2",
      "name": "Название главы",
      "branches": [...]
    }
  ]
}
```

**Примечания:**

- `branches` — ветки перевода. У одной главы может быть несколько переводов от разных команд
- `branch_id` может быть `null` если ветка одна
- `name` — необязательное название главы (часто пусто)

---

### Контент главы

```http
GET /api/manga/{slug}/chapter?number={n}&volume={v}
```

| Параметр | Тип | Описание |
|----------|-----|----------|
| `number` | int | Номер главы |
| `volume` | int | Номер тома |
| `branch_id` | int (опц.) | ID ветки перевода |

**Ответ для ранобе (Site-Id: 3) — текст:**

```json
{
  "data": {
    "id": 789,
    "volume": "1",
    "number": "1",
    "name": "",
    "slug": "chapter-slug",
    "branch_id": null,
    "manga_id": 67008,
    "created_at": "2024-01-15T12:00:00.000000Z",
    "likes_count": 42,
    "teams": [],
    "content": "<p data-paragraph-index=\"1\">Текст первого абзаца...</p><p data-paragraph-index=\"2\">Текст второго абзаца...</p>"
  }
}
```

> ⚠️ Поле `content` — **HTML-строка** с тегами `<p>`. Для получения чистого текста нужно удалить HTML-теги.

**Ответ для манги (Site-Id: 1) — изображения:**

```json
{
  "data": {
    "id": 456,
    "volume": "1",
    "number": "1",
    "content": null,
    "pages": [
      { "slug": 1, "url": "/uploads/manga/i-alone-level-up/chapters/1/001.jpg" },
      { "slug": 2, "url": "/uploads/manga/i-alone-level-up/chapters/1/002.jpg" },
      { "slug": 3, "url": "/uploads/manga/i-alone-level-up/chapters/1/003.jpg" }
    ]
  }
}
```

> Для получения полного URL изображения: `https://img33.imgslib.link/{url}`

---

### Поиск

```http
GET /api/manga?q={query}&site_id[]={site_id}
```

| Параметр | Описание |
|----------|----------|
| `q` | Строка поиска |
| `site_id[]` | `1` для манги, `3` для ранобе |

**Ответ:** массив объектов с краткой информацией (id, name, slug_url, cover...).

---

### Список произведений (каталог)

```http
GET /api/manga?site_id[]={site_id}&sort_by={field}&sort_type={order}&page={page}
```

| Параметр | Значения | По умолчанию |
|----------|----------|-------------|
| `sort_by` | `rate_avg`, `views`, `releaseDate`, `last_episode_at`, `episodes_count` | Популярность |
| `sort_type` | `asc`, `desc` | `desc` |
| `page` | 1, 2, 3... | 1 |
| `rate_min` | число (фильтр мин. рейтинга) | — |

---

### Топ просмотров

```http
GET /api/media/top-views?page={page}&popularity={cat}&time={interval}
```

| Параметр | Значения |
|----------|----------|
| `popularity` | `1` (новые), `2` (набирающие), `3` (популярные) |
| `time` | `day`, `week`, `month` |

---

### Статистика

```http
GET /api/manga/{slug}/stats?bookmarks=true&rating=true
```

Возвращает распределение оценок и кол-во в закладках.

---

## Эндпоинты — Аниме

Аниме использует отдельный префикс `/api/anime/` и `Site-Id: 5`.

### Информация об аниме

```http
GET /api/anime/{slug}?fields[]=summary&fields[]=episodes_count&...
```

**Доступные fields[]:**

```	ext
background, eng_name, otherNames, summary, releaseDate, type_id, caution,
views, close_view, rate_avg, rate, genres, tags, teams, franchise, authors,
publisher, userRating, moderated, metadata, metadata.count,
metadata.close_comments, anime_status_id, time, episodes,
episodes_count, episodesSchedule, shiki_rate
```

**Ответ:**

```json
{
  "data": {
    "id": 19783,
    "name": "Jujutsu Kaisen 2nd Season",
    "rus_name": "Магическая битва 2",
    "slug_url": "19783--jujutsu-kaisen-2nd-season-anime",
    "type": { "label": "TV" },
    "status": { "label": "Вышел" },
    "items_count": { "total": 23, "uploaded": 23 },
    "time": { "formated": "24 мин." },
    "shiki_rate": 8.72,
    "cover": { "default": "..." },
    "genres": [...],
    "teams": [...]
  }
}
```

---



### Поиск аниме

```http
GET /api/anime?q={query}&site_id[]=5
```

---

## Серверы изображений

| Домен | Использование |
|-------|---------------|
| `img33.imgslib.link` | Страницы манги (основной) |
| `cover.imgslib.link` | Обложки |
| `img2.hentaicdn.org` | Альтернативный CDN |

Полный URL изображения: `https://{img_domain}/{page_url}`

---

## Коды ошибок

| HTTP код | Описание |
|----------|----------|
| `200` | Успех |
| `403` | Отсутствует или неверный `Site-Id` / `Referer` |
| `404` | Произведение или глава не найдены |
| `429` | Превышен лимит запросов (заголовок `Retry-After`) |
| `500/502/503` | Ошибка сервера (стоит повторить через несколько секунд) |
| `521` | Cloudflare — сервер недоступен |

---

## Рекомендации

1. **Задержка между запросами** — 0.5–1 секунда. Без неё можно получить 429
2. **Кэширование** — данные о произведении и списки глав меняются редко, стоит кэшировать
3. **Retry** — при 5xx и 521 стоит повторять запрос 2–3 раза с задержкой
4. **Обработка HTML** — контент глав ранобе приходит как HTML (`<p>` теги). Для чистого текста нужна очистка от тегов
5. **branch_id** — если у главы несколько переводов, указывайте `branch_id` при запросе контента

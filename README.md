# AnimeLib Desktop

Native desktop client for animelib.org, built with Rust and egui/eframe. 

## Features
- Fast, lightweight, fully native UI (no Electron/WebView).
- Browse anime catalog with search and pagination.
- View anime details, episodes, and players.
- External video playback via `mpv`.
- Asynchronous image loading and caching.

## Prerequisites
- Rust 1.75+
- `mpv` installed and available in your system's PATH.

## Build & Run
```bash
cargo build --release
cargo run --release
```

## API Notes
The app uses the unofficial `https://api.cdnlibs.org/api/` REST API. No API key is required, and `site_id = 4` is used to fetch anime library data.

## Limitations
- Only `mpv` is currently supported for playback.

## License
MIT

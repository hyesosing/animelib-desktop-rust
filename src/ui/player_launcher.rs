// src/ui/player_launcher.rs
use std::process::Command;

pub fn launch_mpv(url: &str, fallback_site_url: &str) {
    let fixed_url = if url.starts_with("//") {
        format!("https:{}", url)
    } else {
        url.to_string()
    };

    if fixed_url.contains("kodikplayer.com") || fixed_url.contains("rutube.ru") {
        log::info!("Opening web player in browser: {}", fallback_site_url);
        #[cfg(target_os = "windows")]
        let _ = Command::new("cmd").arg("/C").arg("start").arg("").arg(fallback_site_url).spawn();
        
        #[cfg(target_os = "linux")]
        let _ = Command::new("xdg-open").arg(fallback_site_url).spawn();
        
        #[cfg(target_os = "macos")]
        let _ = Command::new("open").arg(fallback_site_url).spawn();
        
        return;
    }

    let paths = vec![
        "mpv",
        "C:\\Program Files\\MPV Player\\mpv.exe",
        "C:\\Program Files\\mpv\\mpv.exe",
        "C:\\ProgramData\\scoop\\apps\\mpv\\current\\mpv.exe",
    ];
    
    let mut success = false;
    for path in paths {
        let result = Command::new(path)
            .arg(&fixed_url)
            .spawn();

        match result {
            Ok(_) => {
                log::info!("Launched mpv using path '{}' for URL: {}", path, url);
                success = true;
                break;
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::error!("Failed to launch mpv at {}: {}", path, e);
                }
            }
        }
    }

    if !success {
        log::error!("Failed to launch mpv: program not found. Make sure mpv is installed and in your PATH.");
    }
}

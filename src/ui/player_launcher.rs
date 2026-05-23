// src/ui/player_launcher.rs
use std::process::Command;

pub fn launch_mpv(url: &str, fallback_site_url: &str, hwnd: Option<isize>) -> Option<std::process::Child> {
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
        
        return None;
    }

    let paths = vec![
        "mpv",
        "C:\\Program Files\\MPV Player\\mpv.exe",
        "C:\\Program Files\\mpv\\mpv.exe",
        "C:\\ProgramData\\scoop\\apps\\mpv\\current\\mpv.exe",
    ];
    
    for path in paths {
        let mut cmd = Command::new(path);
        cmd.arg(&fixed_url);
        
        if let Some(h) = hwnd {
            cmd.arg(format!("--wid={}", h));
            cmd.arg("--force-window=immediate");
            // Also it's often good to set no window border and auto-fit if embedded
            cmd.arg("--no-border");
            cmd.arg("--keep-open=yes"); // keep player open after video ends
            
            // Apply ModernX UI skin
            cmd.arg("--osc=no"); // Disable default OSC
            if let Ok(cwd) = std::env::current_dir() {
                let assets_dir = cwd.join("assets");
                cmd.arg(format!("--config-dir={}", assets_dir.display()));
                
                let script_path = assets_dir.join("modernx.lua");
                if script_path.exists() {
                    cmd.arg(format!("--script={}", script_path.display()));
                } else {
                    log::warn!("ModernX script not found at {}", script_path.display());
                }
            }
            
            // Allow input explicitly (some MPV versions need this when embedded)
            cmd.arg("--input-default-bindings=yes");
            cmd.arg("--input-vo-keyboard=yes");
        }

        let result = cmd.spawn();

        match result {
            Ok(child) => {
                log::info!("Launched mpv using path '{}' for URL: {}", path, url);
                return Some(child);
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    log::error!("Failed to launch mpv at {}: {}", path, e);
                }
            }
        }
    }

    log::error!("Failed to launch mpv: program not found. Make sure mpv is installed and in your PATH.");
    None
}

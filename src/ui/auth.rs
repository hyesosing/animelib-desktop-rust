use std::sync::mpsc;
use std::thread;

pub fn open_auth_window(callback: mpsc::Sender<String>) {
    thread::spawn(move || {
        let exe = std::env::current_exe().unwrap();
        // Spawn a subprocess to run the webview on its main thread.
        // This avoids tao/winit event loop conflicts with eframe.
        if let Ok(output) = std::process::Command::new(exe).arg("--auth-mode").output() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                if let Some(token) = line.strip_prefix("TOKEN:") {
                    let _ = callback.send(token.to_string());
                    return;
                }
            }
        }
        let _ = callback.send(String::new()); // send empty if failed
    });
}

// This function is ONLY called in the subprocess where we have full control over the main thread.
pub fn run_auth_process() {
    use tao::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    use wry::WebViewBuilder;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Авторизация AnimeLib (Решите Captcha)")
        .with_inner_size(tao::dpi::LogicalSize::new(600, 800))
        .build(&event_loop)
        .unwrap();

    let builder = WebViewBuilder::new();
    
    let injection_script = r#"
        setInterval(function() {
            try {
                let authStr = window.localStorage.getItem('auth');
                if (authStr) {
                    let parsed = JSON.parse(authStr);
                    if (parsed && parsed.token && parsed.token.access_token) {
                        let tokenStr = parsed.token.access_token;
                        if (typeof tokenStr === 'string' && tokenStr.length > 40) {
                            window.ipc.postMessage(tokenStr);
                        }
                    }
                }
            } catch (e) {
                console.error(e);
            }
        }, 1000);
    "#;
    
    let proxy = event_loop.create_proxy();
    
    let _webview = builder
        .with_url("https://v5.animelib.org/")
        .with_initialization_script(injection_script)
        .with_ipc_handler(move |msg| {
            // Print the token to stdout so the parent process can read it!
            let body = msg.body().clone();
            println!("TOKEN:{}", body);
            let _ = proxy.send_event(()); // Wake up event loop to exit
        })
        .build(&window)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::UserEvent(_) => {
                // IPC handler woke us up because we got the token!
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

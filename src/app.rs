// src/app.rs
use crate::api::ApiClient;
use crate::cache::ImageCache;
use crate::ui::{anime_page::AnimePageScreen, catalog::CatalogScreen};
use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Clone, PartialEq)]
pub enum Screen {
    Catalog,
    AnimePage(String),
}

pub enum NavAction {
    None,
    GoTo(Screen),
    GoBack,
}

pub struct AnimeLibApp {
    pub hwnd: Option<isize>,
    pub child_hwnd: Option<isize>,
    rt: Arc<Runtime>,
    api_client: Arc<ApiClient>,
    image_cache: Arc<Mutex<ImageCache>>,
    screen_stack: Vec<Screen>,
    
    catalog_screen: CatalogScreen,
    anime_page_screen: Option<AnimePageScreen>,
    
    search_query: String,
    
    auth_rx: Option<std::sync::mpsc::Receiver<String>>,
}

impl AnimeLibApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        crate::ui::theme::setup_custom_theme(&cc.egui_ctx);
        let rt = Arc::new(Runtime::new().expect("Failed to create tokio runtime"));
        let api_client = Arc::new(ApiClient::new());
        let image_cache = Arc::new(Mutex::new(ImageCache::new()));

        if let Ok(json) = std::fs::read_to_string("credentials.json") {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json) {
                if let Some(t) = val.get("token").and_then(|v| v.as_str()) {
                    let mut lock = api_client.auth_token.lock().unwrap();
                    *lock = Some(t.to_string());
                }
            }
        }

        Self {
            hwnd: None,
            child_hwnd: None,
            rt,
            api_client,
            image_cache,
            screen_stack: vec![Screen::Catalog],
            catalog_screen: CatalogScreen::new(),
            anime_page_screen: None,
            search_query: String::new(),
            auth_rx: None,
        }
    }

    fn current_screen(&self) -> &Screen {
        self.screen_stack.last().unwrap_or(&Screen::Catalog)
    }

    fn navigate(&mut self, action: NavAction) {
        match action {
            NavAction::None => {}
            NavAction::GoTo(screen) => {
                if let Screen::AnimePage(ref slug) = screen {
                    self.anime_page_screen = Some(AnimePageScreen::new(slug.clone()));
                } else if let Screen::Catalog = screen {
                    self.anime_page_screen = None;
                    #[cfg(target_os = "windows")]
                    if let Some(h) = self.child_hwnd {
                        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                        unsafe { ShowWindow(h as _, SW_HIDE); }
                    }
                }
                self.screen_stack.push(screen);
            }
            NavAction::GoBack => {
                if self.screen_stack.len() > 1 {
                    self.screen_stack.pop();
                    
                    if let Screen::Catalog = self.current_screen() {
                        self.anime_page_screen = None;
                        #[cfg(target_os = "windows")]
                        if let Some(h) = self.child_hwnd {
                            use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
                            unsafe { ShowWindow(h as _, SW_HIDE); }
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for AnimeLibApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = &self.auth_rx {
            if let Ok(token) = rx.try_recv() {
                if !token.is_empty() {
                    let mut lock = self.api_client.auth_token.lock().unwrap();
                    *lock = Some(token.clone());
                    if let Ok(json) = serde_json::to_string(&serde_json::json!({"token": token})) {
                        let _ = std::fs::write("credentials.json", json);
                    }
                }
                self.auth_rx = None;
            }
        }

        if self.hwnd.is_none() {
            #[cfg(target_os = "windows")]
            {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                if let Ok(handle) = _frame.window_handle() {
                    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
                        let parent = win32.hwnd.get() as isize;
                        self.hwnd = Some(parent);
                        
                        use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS};
                        let class_name: Vec<u16> = "STATIC\0".encode_utf16().collect();
                        unsafe {
                            let child = CreateWindowExW(
                                0,
                                class_name.as_ptr(),
                                std::ptr::null(),
                                WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
                                0, 0, 0, 0, // Starts hidden with zero size
                                parent as _,
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                                std::ptr::null_mut(),
                            );
                            self.child_hwnd = Some(child as isize);
                        }
                    }
                }
            }
        }

                let mut nav_action = NavAction::None;

        // --- PREMIUM FLOATING TOPBAR ---
        egui::TopBottomPanel::top("top_bar")
            .exact_height(60.0)
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(15, 15, 17)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                
                // Drag handle for the whole bar
                let title_bar_response = ui.interact(rect, ui.id().with("top_bar_drag"), egui::Sense::click_and_drag());
                if title_bar_response.is_pointer_button_down_on() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal(|ui| {
                    let side_width = 250.0;
                    
                    // 1. LEFT ITEMS (Logo)
                    ui.allocate_ui_with_layout(egui::vec2(side_width, ui.available_height()), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(24.0);
                        
                        if self.screen_stack.len() > 1 {
                            let back_img = egui::Image::new(egui::include_image!("../assets/icons/back.svg")).tint(egui::Color32::from_rgb(200,200,200)).fit_to_exact_size(egui::vec2(20.0, 20.0));
                            if ui.add(egui::Button::image(back_img).frame(false)).clicked() {
                                nav_action = NavAction::GoBack;
                            }
                            ui.add_space(8.0);
                        }
                        
                        ui.label(egui::RichText::new("AnimeLib").size(22.0).strong().color(egui::Color32::WHITE));
                    });

                    // 2. CENTER ITEMS (Search)
                    let center_width = ui.available_width() - side_width;
                    ui.allocate_ui_with_layout(egui::vec2(center_width, ui.available_height()), egui::Layout::left_to_right(egui::Align::Center).with_main_align(egui::Align::Center), |ui| {
                        // Catalog Button (Circle)
                        let cat_img = egui::Image::new(egui::include_image!("../assets/icons/catalog.svg")).tint(egui::Color32::from_rgb(220,220,220)).fit_to_exact_size(egui::vec2(20.0, 20.0));
                        let catalog_btn = egui::Button::image(cat_img)
                            .fill(egui::Color32::from_rgb(38, 38, 40))
                            .rounding(egui::Rounding::same(22.0))
                            .min_size(egui::vec2(44.0, 44.0));
                        if ui.add(catalog_btn).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                            nav_action = NavAction::GoTo(Screen::Catalog);
                        }
                        
                        ui.add_space(12.0);
                        
                        // Search Bar (Pill)
                        let search_frame = egui::Frame::default()
                            .fill(egui::Color32::from_rgb(38, 38, 40))
                            .rounding(egui::Rounding::same(22.0))
                            .inner_margin(egui::Margin::symmetric(20.0, 12.0));
                            
                        search_frame.show(ui, |ui| {
                            ui.add(egui::Image::new(egui::include_image!("../assets/icons/search.svg")).tint(egui::Color32::from_rgb(150,150,150)).fit_to_exact_size(egui::vec2(16.0, 16.0)));
                            ui.add_space(10.0);
                            
                            let old_visuals = ui.visuals().clone();
                            ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
                            ui.visuals_mut().widgets.hovered.bg_fill = egui::Color32::TRANSPARENT;
                            ui.visuals_mut().widgets.active.bg_fill = egui::Color32::TRANSPARENT;
                            ui.visuals_mut().selection.stroke = egui::Stroke::NONE; // Remove border on focus
                            
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text(egui::RichText::new("Поиск по названию...").color(egui::Color32::from_rgb(140, 140, 145)).size(15.0))
                                    .text_color(egui::Color32::WHITE)
                                    .desired_width(380.0)
                                    .frame(false)
                                    .margin(egui::Margin::same(0.0))
                            );
                            
                            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                self.catalog_screen.set_search(self.search_query.clone());
                                nav_action = NavAction::GoTo(Screen::Catalog);
                            }
                            
                            ctx.set_visuals(old_visuals);
                        });
                    });

                    // 3. RIGHT ITEMS (Window controls & Avatar)
                    ui.allocate_ui_with_layout(egui::vec2(side_width, ui.available_height()), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(24.0);
                        
                        let close_img = egui::Image::new(egui::include_image!("../assets/icons/close.svg")).tint(egui::Color32::from_rgb(150,150,150)).fit_to_exact_size(egui::vec2(14.0, 14.0));
                        if ui.add(egui::Button::image(close_img).frame(false)).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        
                        ui.add_space(8.0);
                        
                        let min_img = egui::Image::new(egui::include_image!("../assets/icons/minimize.svg")).tint(egui::Color32::from_rgb(150,150,150)).fit_to_exact_size(egui::vec2(14.0, 14.0));
                        if ui.add(egui::Button::image(min_img).frame(false)).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        
                        ui.add_space(24.0);
                        
                        let has_token = self.api_client.auth_token.lock().unwrap().is_some();
                        let user_img = egui::Image::new(egui::include_image!("../assets/icons/user.svg")).tint(egui::Color32::from_rgb(220,220,220)).fit_to_exact_size(egui::vec2(32.0, 32.0));
                        let user_btn = egui::Button::image(user_img).frame(false);
                        if has_token {
                            let response = ui.add(user_btn);
                            let popup_id = ui.make_persistent_id("user_dropdown");
                            if response.clicked() {
                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                            }
                            
                            egui::popup::popup_below_widget(ui, popup_id, &response, |ui| {
                                ui.set_min_width(120.0);
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new("Профиль").strong().color(egui::Color32::WHITE));
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);
                                if ui.button(egui::RichText::new("Выйти").color(egui::Color32::from_rgb(255, 80, 80))).clicked() {
                                    let mut lock = self.api_client.auth_token.lock().unwrap();
                                    *lock = None;
                                    let _ = std::fs::remove_file("credentials.json");
                                    ui.memory_mut(|mem| mem.close_popup());
                                }
                                ui.add_space(4.0);
                            });
                        } else {
                            if self.auth_rx.is_some() {
                                ui.label(egui::RichText::new("...").size(24.0).color(egui::Color32::GRAY));
                            } else {
                                if ui.add(user_btn).clicked() {
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    self.auth_rx = Some(rx);
                                    crate::ui::auth::open_auth_window(tx);
                                }
                            }
                        }
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_screen() {
                Screen::Catalog => {
                    self.catalog_screen.render(
                        ui,
                        self.api_client.clone(),
                        self.image_cache.clone(),
                        &mut nav_action,
                        self.rt.clone(),
                    );
                }
                Screen::AnimePage(_) => {
                    if let Some(screen) = &mut self.anime_page_screen {
                        screen.render(
                            ui,
                            self.api_client.clone(),
                            self.image_cache.clone(),
                            &mut nav_action,
                            self.rt.clone(),
                            self.child_hwnd,
                        );
                    }
                }
            }
        });

        self.navigate(nav_action);
    }
}

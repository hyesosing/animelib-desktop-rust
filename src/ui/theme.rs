use eframe::egui::{self, Color32, Rounding, Stroke, Visuals, Margin};

pub fn setup_custom_theme(ctx: &egui::Context) {
    // Load FontAwesome if available
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(font_data) = std::fs::read("assets/fonts/fa-solid-900.ttf") {
        fonts.font_data.insert(
            "FontAwesomeSolid".to_owned(),
            egui::FontData::from_owned(font_data),
        );
        if let Some(vec) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
            vec.push("FontAwesomeSolid".to_owned());
        }
    }
    ctx.set_fonts(fonts);

    let mut visuals = Visuals::dark();

    // ── Color Palette ──────────────────────────────────────────────
    let bg_base     = Color32::from_rgb(14, 14, 16);    // Main background
    let bg_surface  = Color32::from_rgb(24, 24, 28);    // Popups, tooltips, elevated surfaces
    let bg_element  = Color32::from_rgb(34, 34, 38);    // Buttons, inputs, cards
    let bg_hover    = Color32::from_rgb(44, 44, 50);    // Hovered elements
    let bg_active   = Color32::from_rgb(54, 54, 60);    // Pressed elements
    let accent      = Color32::from_rgb(126, 87, 194);  // Purple accent (#7E57C2)
    let text_primary   = Color32::from_rgb(225, 225, 230);
    let text_secondary = Color32::from_rgb(140, 140, 148);
    let border_subtle  = Color32::from_rgba_unmultiplied(255, 255, 255, 10);

    // ── Backgrounds ────────────────────────────────────────────────
    visuals.panel_fill = bg_base;
    visuals.window_fill = bg_surface;
    visuals.faint_bg_color = bg_element;
    visuals.extreme_bg_color = Color32::from_rgb(6, 6, 8);

    // ── Accent / Selection ─────────────────────────────────────────
    visuals.selection.bg_fill = accent;
    visuals.selection.stroke = Stroke::new(1.0, accent);

    // ── Text ───────────────────────────────────────────────────────
    visuals.override_text_color = Some(text_primary);

    // ── Widgets: Noninteractive (labels, separators, frames) ──────
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(255, 255, 255, 5);
    visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, text_secondary);

    // ── Widgets: Inactive (buttons at rest) ───────────────────────
    visuals.widgets.inactive.bg_fill = bg_element;
    visuals.widgets.inactive.weak_bg_fill = bg_element;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.rounding = Rounding::same(8.0);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, text_primary);

    // ── Widgets: Hovered ──────────────────────────────────────────
    visuals.widgets.hovered.bg_fill = bg_hover;
    visuals.widgets.hovered.weak_bg_fill = bg_hover;
    visuals.widgets.hovered.bg_stroke = Stroke::NONE;
    visuals.widgets.hovered.rounding = Rounding::same(8.0);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    // ── Widgets: Active (pressed) ─────────────────────────────────
    visuals.widgets.active.bg_fill = bg_active;
    visuals.widgets.active.weak_bg_fill = bg_active;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.rounding = Rounding::same(8.0);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    // ── Widgets: Open (dropdown/menu open) ────────────────────────
    visuals.widgets.open.bg_fill = bg_hover;
    visuals.widgets.open.weak_bg_fill = bg_hover;
    visuals.widgets.open.bg_stroke = Stroke::NONE;
    visuals.widgets.open.rounding = Rounding::same(8.0);
    visuals.widgets.open.fg_stroke = Stroke::new(1.0, Color32::WHITE);

    // ── Windows & Popups ──────────────────────────────────────────
    visuals.window_rounding = Rounding::same(12.0);
    visuals.window_stroke = Stroke::new(1.0, border_subtle);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 28.0,
        spread: 4.0,
        color: Color32::from_black_alpha(160),
    };
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 6.0),
        blur: 24.0,
        spread: 2.0,
        color: Color32::from_black_alpha(180),
    };

    // ── Misc ──────────────────────────────────────────────────────
    visuals.resize_corner_size = 8.0;
    visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    ctx.set_visuals(visuals);

    // ── Style Overrides ───────────────────────────────────────────
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.window_margin = Margin::same(16.0);
    style.spacing.button_padding = egui::vec2(14.0, 7.0);
    ctx.set_style(style);
}

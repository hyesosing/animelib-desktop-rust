use eframe::egui;

pub struct HomePageScreen {
}

impl HomePageScreen {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("Главная страница (в разработке)");
    }
}

use eframe::egui::Window;

use crate::{EguiView, StatefulText, TextState};

pub struct ErrorWindow<E: std::error::Error> {
    error: E,
}

impl<E: std::error::Error> ErrorWindow<E> {
    pub fn new(error: E) -> Self {
        Self { error }
    }
}

impl<E: std::error::Error> EguiView for ErrorWindow<E> {
    fn show(&mut self, ctx: &eframe::egui::Context, is_open: &mut bool) {
        const TITLE: &str = "Error";
        Window::new(TITLE)
            .resizable(true)
            .collapsible(true)
            .title_bar(true)
            .open(is_open)
            .show(ctx, |ui| self.ui(ui));
    }

    fn ui(&mut self, ui: &mut eframe::egui::Ui) {
        let text = StatefulText::new(
            format!("An error occurred: {}", self.error),
            TextState::Error,
        );
        text.show(ui);
        ui.collapsing("Debug view", |ui| {
            ui.label(format!("{:?}", self.error));
        });
    }
}

use eframe::{egui, epi};

#[derive(Default)]
struct MyEguiApp {
    is_running: bool,
}

impl epi::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let text = if self.is_running { "Stop" } else { "Start" };
            if ui.button(text).clicked() {
                self.is_running = !self.is_running;
            }
        });
    }

    fn name(&self) -> &str {
        "xlogger"
    }
}

fn main() {
    let app = MyEguiApp::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}

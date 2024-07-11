use eframe::{egui, App, NativeOptions};

pub struct Screen;

impl Screen {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl App for Screen {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, world!");
            if ui.button("Click me").clicked() {
                ui.label("You clicked the button!");
            }
        });
    }

}

// rabbitmq_ui/src/main.rs
mod app;

fn main() {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RabbitMQ UI",
        native_options,
        Box::new(|_cc| Box::new(app::App::default())),
    )
    .unwrap();
}

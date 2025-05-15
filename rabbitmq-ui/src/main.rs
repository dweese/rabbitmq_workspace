mod app;

use app::App;
use eframe;
use env_logger;

fn main() -> eframe::Result<()> {
    // Initialize logger
    env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "RabbitMQ UI",
        native_options,
        Box::new(|_cc| Box::new(App::default())),
    )
}
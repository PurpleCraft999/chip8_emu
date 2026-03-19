// mod emulator;

use eframe::{
    NativeOptions,
    egui::{IconData, ViewportBuilder, vec2},
};

use purple8::emulator::window::{EmulatorWindow, MENU_BAR_HEIGHT};

fn main() {
    let window_size = vec2(620., 310. + MENU_BAR_HEIGHT);
    let viewport = ViewportBuilder::default()
        .with_inner_size(window_size)
        .with_min_inner_size(window_size)
        .with_icon(IconData::default());

    let options = NativeOptions {
        viewport,

        ..Default::default()
    };

    eframe::run_native(
        "Purple8",
        options,
        Box::new(|cc| Ok(Box::new(EmulatorWindow::new(cc)))),
    )
    .unwrap();
}

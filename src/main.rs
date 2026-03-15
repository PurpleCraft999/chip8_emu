mod emulator_window;

use eframe::{
    NativeOptions,
    egui::{Vec2, ViewportBuilder},
};

use crate::emulator_window::main_window::{EmulatorWindow, MENU_BAR_HEIGHT};

fn main() {
    let viewport = ViewportBuilder::default()
        .with_inner_size(Vec2::new(620., (32. * 10.) + MENU_BAR_HEIGHT + 20.));

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

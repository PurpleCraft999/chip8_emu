use eframe::egui::{
    self, Color32, ColorImage, MenuBar, Slider, TextureHandle, TextureOptions, TopBottomPanel,
    Widget, containers::menu::MenuConfig,
};
use purple8::{
    Chip8Emulator, SCREEN_HEIGHT, SCREEN_WIDTH, chip8::chip8_emulator_helpers::Chip8Clock,
};

struct EmulatorSettings {
    cycle_speed: u16,
}
impl EmulatorSettings {
    fn new() -> Self {
        Self { cycle_speed: 300 }
    }
}

pub struct EmulatorWindow {
    chip8_emulator: Chip8Emulator,
    chip8_clock: Chip8Clock,
    chip8_screen: TextureHandle,
    settings: EmulatorSettings,
}

impl EmulatorWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            chip8_emulator: Chip8Emulator::init(),
            chip8_clock: Chip8Clock::new(),
            chip8_screen: cc.egui_ctx.load_texture(
                "chip8_screen",
                ColorImage::filled(
                    [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
                    Color32::BLACK,
                ),
                TextureOptions::NEAREST,
            ),
            settings: EmulatorSettings::new(),
        }
    }

    fn chip8_cycle(&mut self) {
        for _ in 0..self.settings.cycle_speed {
            self.chip8_emulator.cycle();
        }
        if self.chip8_clock.tick_chip8_timers() {
            self.chip8_emulator.tick_timers();
        }
    }
    fn chip8_render(&mut self) {
        let display = self.chip8_emulator.get_display();
        let mut pixels = vec![0u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4];

        for (i, &pixel) in display.iter().enumerate() {
            let color: u8 = if pixel == 1 { 255 } else { 0 };
            pixels[i * 4] = color; // R
            pixels[i * 4 + 1] = color; // G
            pixels[i * 4 + 2] = color; // B
            pixels[i * 4 + 3] = 255; // A
        }

        let image = egui::ColorImage::from_rgba_unmultiplied(
            [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
            &pixels,
        );
        self.chip8_screen.set(image, TextureOptions::NEAREST);

        self.chip8_emulator.done_drawing();
    }
}

impl eframe::App for EmulatorWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.chip8_cycle();

        ctx.input(|i| {
            for (index, key) in self.chip8_emulator.get_key_map().iter().enumerate() {
                self.chip8_emulator.set_key(index, i.key_down(*key));
            }

            let droped_files = &i.raw.dropped_files;

            if let Some(file) = &droped_files.first()
                && let Some(path) = &file.path
            {
                let _ = self.chip8_emulator.load_game(path);
            }
        });
        if self.chip8_emulator.get_draw_flag() {
            self.chip8_render();
        }

        TopBottomPanel::top("control bar")
            .exact_height(MENU_BAR_HEIGHT)
            .show(ctx, |ui| {
                MenuBar::new()
                    .config(
                        MenuConfig::new().close_behavior(egui::PopupCloseBehavior::CloseOnClick),
                    )
                    .ui(ui, |ui| {
                        ui.menu_button("file", |ui| {
                            if ui.button("Load").clicked()
                                && let Some(path) = rfd::FileDialog::new()
                                    .set_title("Chip8 game")
                                    .add_filter("ch8 files", &["ch8"])
                                    .pick_file()
                            {
                                self.chip8_emulator.load_game(&path).unwrap();
                            }
                            //  if ui.button("play sound").clicked(){
                            //     self.chip8_emulator.load_bytes_into_memory(&[0x60,0x03,0xF0,0x18,0x12,0x04]);
                            //  }
                            //  0x204
                        });
                        ui.menu_button("settings", |ui| {
                            ui.horizontal(|ui| {
                                ui.label("emulation speed");
                                Slider::new(&mut self.settings.cycle_speed, 0..=1000).ui(ui);
                            });
                            ui.menu_button("key binds", |_| {
                                // ui.
                            })
                        });
                    })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let image = egui::Image::new(&self.chip8_screen);
            ui.add(image.fit_to_exact_size(ui.available_size()));
        });

        self.chip8_emulator.done_drawing();
    }
}
pub(crate) const MENU_BAR_HEIGHT: f32 = 25.;

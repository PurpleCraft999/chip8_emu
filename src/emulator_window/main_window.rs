use eframe::egui::{
    self, Color32, ColorImage, Context, Frame, Key, MenuBar, Pos2, Rect, ScrollArea, Slider,
    TextEdit, TextureHandle, TextureOptions, TopBottomPanel, Widget, Window,
    containers::menu::MenuConfig, vec2,
};
use purple8::{
    Chip8Emulator, SCREEN_HEIGHT, SCREEN_WIDTH, chip8::chip8_emulator_helpers::Chip8Clock,
};

struct EmulatorSettings {
    cycle_speed: u16,
    key_map: [Key; 16],
    key_binds_window_open: bool,
}
impl Default for EmulatorSettings {
    fn default() -> Self {
        Self {
            cycle_speed: 300,
            key_binds_window_open: false,
            key_map: DEFAULT_KEY_MAP,
        }
    }
}
const DEFAULT_KEY_MAP: [Key; 16] = [
    Key::Num0,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Num8,
    Key::Num9,
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
];

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
            settings: EmulatorSettings::default(),
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

        let image = image_from_chip8_display(display);
        self.chip8_screen.set(image, TextureOptions::NEAREST);

        self.chip8_emulator.done_drawing();
    }
    fn key_binds_window(&mut self, ctx: &Context) {
        Window::new("Key Binds")
            .collapsible(false)
            .default_width(30.)
            .open(&mut self.settings.key_binds_window_open)
            .default_height(200.)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (i, key) in self.settings.key_map.iter_mut().enumerate() {
                        let mut key_name = key.name().to_string();
                        if *key == Key::Space {
                            key_name = "".to_string();
                        }
                        TextEdit::singleline(&mut key_name)
                            .hint_text(format!("key:{i:X}"))
                            .char_limit(1)
                            .ui(ui);
                        if let Some(new_key) = Key::from_name(&key_name) {
                            *key = new_key
                        } else if key_name.is_empty() {
                            *key = Key::Space
                        } else {
                            println!("could not set key")
                        }
                    }
                });
            });
    }
}

impl eframe::App for EmulatorWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        self.chip8_cycle();

        ctx.input(|input| {
            for (index, key) in self.settings.key_map.iter().enumerate() {
                self.chip8_emulator.set_key(index, input.key_down(*key));
            }

            let droped_files = &input.raw.dropped_files;

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
                            if ui.button("KeyBinds").clicked() {
                                self.settings.key_binds_window_open = true;
                            }
                        });
                    })
            });
        self.key_binds_window(ctx);

        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(Color32::DARK_GRAY))
            .show(ctx, |ui| {
                let image = egui::Image::new(&self.chip8_screen);
                let viewport_rect = ctx.viewport_rect();
                let bottom_right = vec2(viewport_rect.width(), viewport_rect.height());

                ui.put(
                    Rect {
                        min: Pos2::new(0., MENU_BAR_HEIGHT),
                        max: bottom_right.to_pos2(),
                    },
                    image.fit_to_exact_size(bottom_right),
                );
            });

        self.chip8_emulator.done_drawing();
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::BLACK.to_normalized_gamma_f32()
    }
}
pub(crate) const MENU_BAR_HEIGHT: f32 = 20.;

fn image_from_chip8_display(display: &[u8; 2048]) -> ColorImage {
    let mut pixels = vec![0u8; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize * 4];

    for (i, &pixel) in display.iter().enumerate() {
        let color: u8 = if pixel == 1 { 255 } else { 0 };
        pixels[i * 4] = color;
        pixels[i * 4 + 1] = color;
        pixels[i * 4 + 2] = color;
        pixels[i * 4 + 3] = 255;
    }

    egui::ColorImage::from_rgba_unmultiplied(
        [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
        &pixels,
    )
}

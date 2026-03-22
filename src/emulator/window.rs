use std::{fmt::Display, fs, path::Path};

use crate::{
    ChipEmulator, SCREEN_HEIGHT, SCREEN_WIDTH,
    chip8::chip8_emulator::{Chip8Clock, Chip8Emulator},
    emulator::Chip,
    super_chip::SuperChipEmulator,
};
use eframe::egui::{
    self, Color32, ColorImage, Context, Frame, Key, MenuBar, Pos2, Rect, ScrollArea, Slider,
    TextEdit, TextureHandle, TextureOptions, TopBottomPanel, Widget, Window,
    containers::menu::MenuConfig, vec2,
};

struct EmulatorSettings {
    cycle_speed: u16,
    key_map: KeyMap,
    key_binds_window_open: bool,
    debug_window_open: bool,
    pause: bool,
}
impl Default for EmulatorSettings {
    fn default() -> Self {
        Self {
            cycle_speed: 300,
            key_binds_window_open: false,
            key_map: OCTO_KEY_MAP,
            debug_window_open: false,
            pause: false,
        }
    }
}
///copy the methods from ChipEmulator to the Emulator enum
macro_rules! copy_methods{
    (
        //allows for both self and mut self to be valid
        $self:ty,

        $(
            //method name
            $method:ident
            //generic definition and type
            $(<$($generic_name:ident:$generic_type:path),+>)?
            //arguments if present
            ($($arg_name:ident: $arg_type:ty),*)
            //return type
            $(-> $return:ty)?
        ),+
            //allow comma at end
            $(,)?
        )=>{
        $(

            fn $method$
                //place the genric before the args
                (<$($generic_name:$generic_type),+>)?
                //self and args
                (self:$self,$($arg_name: $arg_type),*)
                //return type
                $(-> $return)? {
                //calls the methods on the chips
                match self{
                    Self::Chip8(chip) => chip.$method($($arg_name),*),
                    Self::SuperChip(chip) => chip.$method($($arg_name),*),
                }
            }
        )+
    }
}

enum Emulator {
    Chip8(Box<ChipEmulator<Chip8Emulator>>),
    SuperChip(Box<ChipEmulator<SuperChipEmulator>>),
}
impl Emulator {
    fn new_chip8() -> Self {
        Self::Chip8(Box::new(ChipEmulator::new(Chip8Emulator::new())))
    }
    fn new_super_chip() -> Self {
        Self::SuperChip(Box::new(ChipEmulator::new(SuperChipEmulator::new())))
    }
    copy_methods! {&Self,play_sound(),get_display()->&[u8],get_draw_flag()->bool,get_display_size()->(usize,usize),get_v(registry:u8)->u8,get_memory(addr: u16) -> u8,clone_emulator<T:Chip>(new_chip:T)->ChipEmulator<T>}
    copy_methods! {&mut Self,cycle(),tick_timers(),set_key(index:usize,state:bool),set_draw_flag(flag:bool),set_memory(a:usize,v:u8),load_bytes_into_memory(bytes: &[u8])}
}

impl Display for Emulator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Chip8(_) => "chip8 emulator",
            Self::SuperChip(_) => "super chip emulator",
        };

        write!(f, "{name}")
    }
}
type KeyMap = [Key; 16];
const DEFAULT_KEY_MAP: KeyMap = [
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

const OCTO_KEY_MAP: KeyMap = [
    Key::X,    //0
    Key::Num1, //1
    Key::Num2, //2
    Key::Num3, //3
    Key::Q,    //4
    Key::W,    //5
    Key::E,    //6
    Key::A,    //7
    Key::S,    //8
    Key::D,    //9
    Key::Z,    //A
    Key::C,    //B
    Key::Num4, //C
    Key::R,    //D
    Key::F,    //E
    Key::V,    //F
];

pub struct EmulatorWindow {
    emulator: Emulator,
    chip8_clock: Chip8Clock,
    chip8_screen: TextureHandle,
    settings: EmulatorSettings,
}

impl EmulatorWindow {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            emulator: Emulator::new_chip8(),
            chip8_clock: Chip8Clock::new(),
            chip8_screen: cc.egui_ctx.load_texture(
                "chip8_screen",
                ColorImage::filled(
                    [SCREEN_WIDTH as usize * 2, SCREEN_HEIGHT as usize * 2],
                    Color32::BLACK,
                ),
                TextureOptions::NEAREST,
            ),
            settings: EmulatorSettings::default(),
        }
    }

    fn chip8_cycle(&mut self) {
        if self.settings.pause {
            return;
        }
        for _ in 0..self.settings.cycle_speed {
            self.emulator.cycle();
        }
        if self.chip8_clock.tick_chip8_timers() {
            self.emulator.tick_timers();
        }
    }
    fn chip8_render(&mut self) {
        let display = self.emulator.get_display();
        let (width, height) = self.emulator.get_display_size();
        let image = image_from_chip8_display(display, width, height);

        self.chip8_screen.set(image, TextureOptions::NEAREST);

        self.emulator.set_draw_flag(false);
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
    fn debug_window(&mut self, ctx: &Context) {
        Window::new("Debug")
            .collapsible(false)
            // .default_width(60.)
            .open(&mut self.settings.debug_window_open)
            // .default_height(100.)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for i in 0..16 {
                        ui.label(i.to_string() + ":" + &self.emulator.get_v(i).to_string());
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("pause").clicked() {
                        self.settings.pause = true;
                    }
                    if ui.button("advance cycle").clicked() {
                        self.emulator.cycle();
                    }
                    if ui.button("unpause").clicked() {
                        self.settings.pause = false;
                    }
                });
            });
    }
    fn swap_emu(&mut self) {
        self.emulator = match self.emulator {
            Emulator::Chip8(_) => Emulator::SuperChip(Box::new(
                self.emulator.clone_emulator(SuperChipEmulator::new()),
            )),
            Emulator::SuperChip(_) => {
                Emulator::Chip8(Box::new(self.emulator.clone_emulator(Chip8Emulator::new())))
            }
        };
    }
    fn load_game(&mut self, path: &Path) {
        let bytes = fs::read(path).unwrap();

        // bytes.iter().any(|byte|byte.)

        self.emulator.load_bytes_into_memory(&bytes);
    }
}

impl eframe::App for EmulatorWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.chip8_cycle();

        ctx.input(|input| {
            for (index, key) in self.settings.key_map.iter().enumerate() {
                self.emulator.set_key(index, input.key_down(*key));
            }

            let droped_files = &input.raw.dropped_files;

            if let Some(file) = &droped_files.first()
                && let Some(path) = &file.path
            {
                self.load_game(path);
            }
        });
        if self.emulator.get_draw_flag() {
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
                                self.load_game(&path);
                            }
                            if ui.button("debug").clicked() {
                                self.settings.debug_window_open = true;
                            }
                            // if ui.button("tests").clicked() {
                            //     self.emulator.load_game(Path::new("5-quirks.ch8")).unwrap();

                            //     self.emulator.set_memory(0x1FF, 2);
                            // }
                            // if ui.button("draw test").clicked() {
                            //     self.emulator.load_game(Path::new("d.ch8")).unwrap();
                            // }

                            // if ui.button("play sound").clicked() {
                            //     self.emulator
                            //         .load_bytes_into_memory(&[0x60, 0x03, 0xF0, 0x18, 0x12, 0x04]);
                            // }
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
                            // if ui.button("print screen").clicked() {
                            //     println!("{:?}",self.emulator.get_display());
                            // }
                            if ui
                                .button(
                                    "switch emulator | current: ".to_owned()
                                        + &self.emulator.to_string(),
                                )
                                .clicked()
                            {
                                self.swap_emu();
                            }
                        });
                    })
            });
        self.key_binds_window(ctx);
        self.debug_window(ctx);
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

        // self.emulator.done_drawing();
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::BLACK.to_normalized_gamma_f32()
    }
}
pub const MENU_BAR_HEIGHT: f32 = 20.;

fn image_from_chip8_display(display: &[u8], width: usize, height: usize) -> ColorImage {
    let mut pixels = vec![0; display.len() * 4];

    for (i, &pixel) in display.iter().enumerate() {
        let color: u8 = if pixel == 1 { 255 } else { 0 };
        pixels[i * 4] = color;
        pixels[i * 4 + 1] = color;
        pixels[i * 4 + 2] = color;
        pixels[i * 4 + 3] = 255;
    }

    egui::ColorImage::from_rgba_unmultiplied([width, height], &pixels)
}

use crate::{Chip8Opcode, emulator::Chip};

pub(crate) const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];
pub const CHIP8_SCREEN_SIZE: usize = 2048;
pub struct Chip8Emulator {
    ///64 by 32 screen with black or white pixels
    display: [u8; CHIP8_SCREEN_SIZE],
}

impl Chip for Chip8Emulator {
    type Opcode = Chip8Opcode;
    fn get_display(&self) -> &[u8] {
        &self.display
    }
    fn get_display_mut(&mut self) -> &mut [u8] {
        &mut self.display
    }
    fn resize_screen(&mut self, _size: usize) {
        panic!("screen cannot be resized")
    }
    fn new() -> Self {
        Self {
            display: [0; CHIP8_SCREEN_SIZE],
        }
    }
}
use std::time::{Duration, Instant};

use rodio::{MixerDeviceSink, Player, Source, source::SquareWave};
pub struct Chip8Clock {
    last_ran: Instant,
    speed: Duration,
}
impl Chip8Clock {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            last_ran: Instant::now(),
            speed: Duration::from_micros(16667),
        }
    }
    ///returns if chip8 timers should tick
    pub fn tick_chip8_timers(&mut self) -> bool {
        if self.last_ran.elapsed() >= self.speed {
            self.last_ran = Instant::now();
            true
        } else {
            false
        }
    }
}

pub(crate) struct Chip8Audio {
    player: Player,
    mixer: MixerDeviceSink,
}
impl Chip8Audio {
    pub fn new() -> Self {
        let mut mixer = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        mixer.log_on_drop(false);
        let player = Player::connect_new(mixer.mixer());

        player.set_volume(0.01);

        Self { player, mixer }
    }
    pub fn append<S: Source + Send + 'static>(&self, source: S) {
        self.player.append(source);
    }
    pub fn pause(&self) {
        self.player.stop();
    }
    pub fn play(&self) {
        let sound = SquareWave::new(880.);
        self.player.append(sound);
    }
}

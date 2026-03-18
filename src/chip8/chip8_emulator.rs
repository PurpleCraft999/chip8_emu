use crate::{Chip8OpCode, emulator::Chip};

pub struct Chip8Emulator {
    ///64 by 32 screen with black or white pixels
    display: [u8; 2048],
}
impl Chip8Emulator {
    pub fn new() -> Self {
        Self { display: [0; 2048] }
    }
}

impl Chip for Chip8Emulator {
    type OpcodeType = Chip8OpCode;
    fn get_display(&self) -> &[u8] {
        &self.display
    }
    fn get_display_mut(&mut self) -> &mut [u8] {
        &mut self.display
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

use std::time::{Duration, Instant};

use rodio::{MixerDeviceSink, Player, Source, source::SquareWave};
pub struct Chip8Clock {
    last_ran: Instant,
    speed: Duration,
}
impl Chip8Clock {
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
        let mixer = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = Player::connect_new(mixer.mixer());
        player.set_volume(0.01);
        let sound = SquareWave::new(880.);
        player.append(sound);

        Self { player, mixer }
    }
    pub fn append<S: Source + Send + 'static>(&self, source: S) {
        self.player.append(source);
    }
    pub fn pause(&self) {
        self.player.pause();
    }
    pub fn play(&self) {
        self.player.play();
    }
}

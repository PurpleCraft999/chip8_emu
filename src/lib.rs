#![allow(dead_code)]
pub mod chip8;
pub use chip8::{
    // chip8_emulator::Chip8Emulator,
    chip8_opcode::{Chip8OpCode, UnkownOpCodeErr},
};
pub mod emulator;
pub mod super_chip;
pub use emulator::ChipEmulator;
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;

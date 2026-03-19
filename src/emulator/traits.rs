use crate::{ChipEmulator, UnkownOpCodeErr};

pub trait Chip {
    type Opcode: Opcode;
    fn get_display(&self) -> &[u8];
    fn get_display_mut(&mut self) -> &mut [u8];
    fn resize_screen(&mut self, size: usize);
    fn new() -> Self;
}

pub trait Opcode: Sized {
    fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr>;
    fn execute_opcode<C: Chip>(self, emu: &mut ChipEmulator<C>) -> bool;
    fn useless_opcode() -> Self;
}

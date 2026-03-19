use crate::{emulator::Chip, super_chip::SuperChipOpcode};
pub struct SuperChipEmulator {
    display: Vec<u8>,
}

impl Chip for SuperChipEmulator {
    type Opcode = SuperChipOpcode;
    fn get_display(&self) -> &[u8] {
        &self.display
    }
    fn get_display_mut(&mut self) -> &mut [u8] {
        &mut self.display
    }
    fn resize_screen(&mut self, size: usize) {
        self.display.fill(0);
        self.display.resize(size, 0);
    }
    fn new() -> Self {
        Self {
            display: vec![0; 2048],
        }
    }
}

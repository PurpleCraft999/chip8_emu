use crate::{
    ChipEmulator,
    chip8::chip8_emulator::CHIP8_SCREEN_SIZE,
    emulator::Chip,
    super_chip::{SuperChipEmulator, super_chip_emulator::SUPER_CHIP_LARGE_SCREEN_SIZE},
};

fn super_chip_test_helper(cycle_count: u8, bytes: &[u8]) -> ChipEmulator<SuperChipEmulator> {
    chip_test_helper!(SuperChipEmulator, bytes, cycle_count)
}

#[test]
fn test_raise_screen() {
    //raises the screen resolution
    let chip = super_chip_test_helper(1, &[0x0, 0xFF]);
    assert_eq!(SUPER_CHIP_LARGE_SCREEN_SIZE, chip.get_display().len())
}
#[test]
fn test_lower_screen() {
    //raises the screen resolution then lowers it
    let chip = super_chip_test_helper(2, &[0x0, 0xFF, 0x0, 0xFE]);
    assert_eq!(CHIP8_SCREEN_SIZE, chip.get_display().len())
}

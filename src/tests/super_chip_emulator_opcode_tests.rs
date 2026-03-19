use crate::{ChipEmulator, emulator::Chip, super_chip::SuperChipEmulator};

fn super_chip_test_helper(cycle_count: u8, bytes: &[u8]) -> ChipEmulator<SuperChipEmulator> {
    chip_test_helper!(SuperChipEmulator, bytes, cycle_count)
}

#[test]
fn test_raise_screen() {
    //raises the screen resolution
    let chip = super_chip_test_helper(1, &[0x0, 0xFF]);
    assert_eq!(8192, chip.get_display().len())
}
#[test]
fn test_lower_screen() {
    //raises the screen resolution then lowers it
    let chip = super_chip_test_helper(2, &[0x0, 0xFF, 0x0, 0xFE]);
    assert_eq!(2048, chip.get_display().len())
}

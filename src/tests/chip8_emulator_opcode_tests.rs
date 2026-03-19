use crate::{chip8::chip8_emulator::Chip8Emulator, emulator::{Chip, ChipEmulator}};
///make and run the given bytes for the given cycles
///
/// it is an inclusive loop
fn chip8_test_helper(cycle_count: u8, bytes: &[u8]) -> ChipEmulator<Chip8Emulator> {
    let mut chip8 = ChipEmulator::new(Chip8Emulator::new());
    chip8.load_bytes_into_memory(bytes);
    for _ in 0..cycle_count {
        chip8.cycle();
    }
    chip8
}

#[test]
fn single_byte_load() {
    let chip8 = chip8_test_helper(0, &[0x48]);

    let mut correct_bytes = [0; 4096];
    correct_bytes[0x200] = 0x48;
    assert_eq!(correct_bytes, chip8.memory)
}

#[test]
fn two_byte_load() {
    let chip8 = chip8_test_helper(0, &[0x53, 0x94]);

    let mut correct_bytes = [0; 4096];
    correct_bytes[512] = 0x53;
    correct_bytes[513] = 0x94;
    assert_eq!(correct_bytes, chip8.memory)
}

#[test]
fn clear_screen() {
    //loads the clear screen command

    let mut chip8 = chip8_test_helper(0, &[0x00, 0xE0]);
    //set random areas to 1 to see if they get cleared
    chip8.get_display_mut()[0] = 1;
    chip8.get_display_mut()[1032] = 1;
    chip8.get_display_mut()[2020] = 1;
    let len = chip8.get_display().len() - 1;
    chip8.get_display_mut()[len] = 1;
    chip8.cycle();
    assert_eq!([0; 2048], chip8.get_display())
}

#[test]
fn set_v_register() {
    //set reg 1 to 0x08
    let chip8 = chip8_test_helper(1, &[0x61, 0x08]);
    assert_eq!(0x08, chip8.get_v(1))
}

#[test]
fn load_index_registry() {
    //set i reg to 0x250
    let chip8 = chip8_test_helper(1, &[0xA2, 0x50]);
    assert_eq!(0x250, chip8.index_register)
}

#[test]
fn jump() {
    //jump to memory 0xF60
    let chip8 = chip8_test_helper(1, &[0x1F, 0x60]);
    assert_eq!(0xF60, chip8.program_counter)
}

#[test]
fn add() {
    //set reg 0 to 1 then add 0x80 add 0x8 to registry 0
    let chip8 = chip8_test_helper(2, &[0x60, 0x1, 0x70, 0x80]);

    assert_eq!(0x81, chip8.get_v(0))
}

#[test]
fn display() {
    //set i to 0x204(where sprite is) then draw at x=0 y=1 for 5 rows
    let chip8 = chip8_test_helper(2, &[0xA2, 0x04, 0xD0, 0x15, 0x88, 0x50, 0xF8, 0xA8, 0x70]);
    let mut correct_display = [0; 2048];
    //set the test display
    correct_display[0] = 1;
    correct_display[4] = 1;
    correct_display[65] = 1;
    correct_display[67] = 1;
    correct_display[128] = 1;
    correct_display[129] = 1;
    correct_display[130] = 1;
    correct_display[131] = 1;
    correct_display[132] = 1;
    correct_display[192] = 1;
    correct_display[194] = 1;
    correct_display[196] = 1;
    correct_display[257] = 1;
    correct_display[258] = 1;
    correct_display[259] = 1;
    //execute all the needed instructions
    assert_eq!(correct_display, chip8.get_display())
}

#[test]
fn skip_if_register_eq_value() {
    //set v0 to 1 then check if it equals one
    let chip8 = chip8_test_helper(
        2,
        &[
            0x60, 0x01, 0x30, 0x01, //this should never run
            0x62, 0x05,
        ],
    );
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}

#[test]
fn skip_if_register_not_eq_value() {
    //set v0 to 1 the check if != to 0
    let chip8 = chip8_test_helper(
        2,
        &[
            0x60, 0x01, 0x40, 0x00, //this should never run
            0x62, 0x05,
        ],
    );
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}

#[test]
fn skip_if_registers_eq() {
    //set v0 and v1 to 1 then check if equal
    let chip8 = chip8_test_helper(
        3,
        &[
            0x60, 0x01, 0x61, 0x01, 0x50, 0x10, //this should never run
            0x62, 0x05,
        ],
    );
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}

#[test]
fn skip_if_registers_not_eq() {
    //set v0 and v1 to 1 then check if equal
    let chip8 = chip8_test_helper(
        3,
        &[
            0x60, 0x01, 0x61, 0x02, 0x90, 0x10, //this should never run
            0x62, 0x05,
        ],
    );
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}

#[test]
fn set_register_eq() {
    //set register 1 to 0x32 then set reg 0 the the value of reg 1
    let chip8 = chip8_test_helper(2, &[0x61, 0x32, 0x80, 0x10]);
    assert_eq!(chip8.get_v(0), chip8.get_v(1))
}

#[test]
fn call() {
    //make a call to a mem value
    let chip8 = chip8_test_helper(2, &[0x22, 0x04, 0x0, 0x0, 0x60, 0x01]);
    assert_eq!(chip8.get_v(0), 1);
    assert_eq!(chip8.stack_pointer, 1);
    assert_eq!(chip8.program_counter, 0x206)
}

#[test]
fn opcode_return() {
    //does a call to 0x204 then sets reg 1 to 5 then returns and sets reg 0 to 1
    let chip8 = chip8_test_helper(4, &[0x22, 0x04, 0x60, 0x01, 0x61, 0x05, 0x00, 0xEE]);
    assert_eq!(chip8.get_v(1), 5);
    assert_eq!(chip8.get_v(0), 1);
}

#[test]
fn test_or_logic() {
    // set reg 0 to 5, reg 1 to 0xA then preform bitwise or
    let chip8 = chip8_test_helper(3, &[0x60, 0x05, 0x61, 0x0A, 0x80, 0x11]);
    assert_eq!(0x0F, chip8.get_v(0));
}

#[test]
fn test_and_logic() {
    //loads reg 0 with 0xC and reg 1 with 0x06 then preforms bit and on them
    let chip8 = chip8_test_helper(3, &[0x60, 0x0C, 0x61, 0x06, 0x80, 0x12]);
    assert_eq!(0x04, chip8.get_v(0));
}

#[test]
fn test_xor_logic() {
    // set reg 0 to 0xFF the reg 1 to 0xAA the preforms bit xor
    let chip8 = chip8_test_helper(3, &[0x60, 0xFF, 0x61, 0xAA, 0x80, 0x13]);
    assert_eq!(0x55, chip8.get_v(0));
}

///ai
#[test]
fn test_add_reg_8xy4() {
    // 60FE -> V0 = 254 (0xFE)
    // 6103 -> V1 = 3   (0x03)
    // 8014 -> V0 = V0 + V1 (Should result in 257, so V0=1 and VF=1)
    let chip8 = chip8_test_helper(3, &[0x60, 0xFE, 0x61, 0x03, 0x80, 0x14]);

    assert_eq!(0x01, chip8.get_v(0)); // 254 + 3 = 257 (wraps to 1)
    assert_eq!(1, chip8.get_v(0xF)); // Carry flag should be set
}

///ai
#[test]
fn test_sub_reg_8xy5() {
    // 600A -> V0 = 10
    // 6105 -> V1 = 5
    // 8015 -> V0 = V0 - V1 (10 - 5 = 5, VF=1 because NO borrow)
    let chip8 = chip8_test_helper(3, &[0x60, 0x0A, 0x61, 0x05, 0x80, 0x15]);

    assert_eq!(0x05, chip8.get_v(0));
    assert_eq!(1, chip8.get_v(0xF)); // VF=1 means no borrow (result > 0)
}

///ai
#[test]
fn test_sub_reg_yx_8xy7() {
    // 6005 -> V0 = 5
    // 610A -> V1 = 10
    // 8017 -> V0 = V1 - V0 (10 - 5 = 5, VF=1 because NO borrow)
    let chip8 = chip8_test_helper(3, &[0x60, 0x05, 0x61, 0x0A, 0x80, 0x17]);

    assert_eq!(0x05, chip8.get_v(0));
    assert_eq!(1, chip8.get_v(0xF));
}

///ai
#[test]
fn test_shr_8xy6() {
    // 6003 -> V0 = 3 (0000 0011)
    // 8006 -> V0 = V0 >> 1 (Shift right)
    let chip8 = chip8_test_helper(2, &[0x60, 0x03, 0x80, 0x06]);

    assert_eq!(0x01, chip8.get_v(0)); // 3 >> 1 = 1
    assert_eq!(1, chip8.get_v(0xF)); // LSB was 1, so VF = 1
}

///ai
#[test]
fn test_shl_8xye() {
    // 6080 -> V0 = 128 (1000 0000)
    // 800E -> V0 = V0 << 1 (Shift left)
    let chip8 = chip8_test_helper(2, &[0x60, 0x80, 0x80, 0x0E]);

    assert_eq!(0x00, chip8.get_v(0)); // 128 << 1 = 256 (wraps to 0)
    assert_eq!(1, chip8.get_v(0xF)); // MSB was 1, so VF = 1
}

#[test]
fn load_memory_into_registries() {
    //sets i to 0x204 then loads all memory in the the registry
    let chip8 = chip8_test_helper(2, &[0xA2, 0x04, 0xF1, 0x65, 0x43, 0x34]);

    assert_eq!(chip8.get_v(0), 0x43);
    assert_eq!(chip8.get_v(1), 0x34);
    assert_eq!(chip8.index_register, 0x206);
}

#[test]
fn load_memory_from_registries() {
    //load reg 0 to 0x15 then reg 1 to 0x63 then sets i to 0x330 then loads the values from memory into the registries
    let chip8 = chip8_test_helper(4, &[0x60, 0x15, 0x61, 0x63, 0xA3, 0x30, 0xF1, 0x55]);

    assert_eq!(chip8.memory[0x330], 0x15);
    assert_eq!(chip8.memory[0x331], 0x63);
    assert_eq!(chip8.index_register, 0x332);
}

#[test]
fn store_vx_as_binary_three_digit() {
    //set i to 0x100 then set v0 to 0x68 the store v0 as 104 in memory
    let chip8 = chip8_test_helper(3, &[0xA1, 0x00, 0x60, 0x68, 0xF0, 0x33]);
    assert_eq!(chip8.memory[0x100], 1);
    assert_eq!(chip8.memory[0x101], 0);
    assert_eq!(chip8.memory[0x102], 4);
}

#[test]
fn store_vx_as_binary_double_digit() {
    //set i to 0x100 then set v0 to 0x68 the store v0 as 96 in memory
    let chip8 = chip8_test_helper(3, &[0xA1, 0x00, 0x60, 0x60, 0xF0, 0x33]);
    assert_eq!(chip8.memory[0x100], 0);
    assert_eq!(chip8.memory[0x101], 9);
    assert_eq!(chip8.memory[0x102], 6);
}

#[test]
fn store_vx_as_binary_single_digit() {
    //set i to 0x100 then set v0 to 0x68 the store v0 as 5 in memory
    let chip8 = chip8_test_helper(3, &[0xA1, 0x00, 0x60, 0x05, 0xF0, 0x33]);
    assert_eq!(chip8.memory[0x100], 0);
    assert_eq!(chip8.memory[0x101], 0);
    assert_eq!(chip8.memory[0x102], 5);
}
#[test]
fn add_assign_i() {
    //set i and v0 to 0x40 then adds them
    let chip8 = chip8_test_helper(3, &[0xA0, 0x40, 0x60, 0x40, 0xF0, 0x1E]);
    assert_eq!(chip8.index_register, 128);
}
#[test]
fn skip_if_not_pressed_while_pressed() {
    //the first opcode is so it checks at the correct time and not after it has been ran
    //check if key 0 is down then skips the next instruction
    let mut chip8 = chip8_test_helper(0, &[0x00, 0x0, 0xE0, 0xA1, 0x61, 0x5]);
    chip8.set_key(0, true);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    assert_eq!(chip8.get_v(1), 5);
}
#[test]
fn skip_if_not_pressed_while_unpressed() {
    //skips the next instruction because it will never be pressed
    let chip8 = chip8_test_helper(2, &[0xE0, 0xA1, 0x60, 0x5]);
    assert_eq!(chip8.get_v(1), 0);
}

#[test]
fn skip_if_pressed_while_pressed() {
    //the first opcode is so it checks at the correct time and not after it has been ran
    let mut chip8 = chip8_test_helper(0, &[0xE0, 0x9E, 0x61, 0x5]);
    chip8.set_key(0, true);
    chip8.cycle();
    chip8.cycle();
    // chip8.cycle();
    assert_eq!(chip8.get_v(1), 0);
}
#[test]
fn skip_if_pressed_while_unpressed() {
    //skips the next instruction because it will never be pressed
    let chip8 = chip8_test_helper(2, &[0xE0, 0x9E, 0x60, 0x5]);
    assert_eq!(chip8.get_v(1), 0);
}
#[test]
fn wait_for_key_press_without_press() {
    //wait forever for a keypress
    let chip8 = chip8_test_helper(100, &[0xF0, 0x0A, 0x61, 0x5]);
    assert_ne!(chip8.get_v(1), 5);
}
#[test]
fn wait_for_key_press_with_press() {
    let mut chip8 = chip8_test_helper(5, &[0xF0, 0x0A, 0x61, 0x5]);
    chip8.set_key(8, true);
    chip8.cycle();
    chip8.set_key(8, false);
    chip8.cycle();
    chip8.cycle();
    assert_eq!(chip8.get_v(1), 5);
    assert_eq!(chip8.get_v(0), 8);
}
#[test]
fn set_delay_timer() {
    let chip8 = chip8_test_helper(2, &[0x60, 0x50, 0xF0, 0x15]);
    assert_eq!(chip8.delay_timer, 0x50);
}
#[test]
fn set_vx_to_delay() {
    let chip8 = chip8_test_helper(2, &[0x60, 0x50, 0xF0, 0x07]);
    assert_eq!(chip8.get_v(0), chip8.delay_timer);
}
#[test]
fn set_sound_timer() {
    let chip8 = chip8_test_helper(2, &[0x60, 0x50, 0xF0, 0x18]);
    assert_eq!(chip8.sound_timer, 0x50);
}
#[test]
fn jump_plus_v0() {
    let chip8 = chip8_test_helper(2, &[0x60, 0x50, 0xB2, 0x40]);
    assert_eq!(chip8.program_counter, 0x290);
}

#[test]
fn get_font_start() {
    for i in 0..16 {
        let command = u8::from_str_radix(format!("F{i:X}").as_str(), 16).unwrap();
        let chip8 = chip8_test_helper(1, &[command, 0x29]);
        assert_eq!(chip8.index_register, i * 5);
    }
}

/*
need tests
    ///0xC
    Random { registry: u8, value: u8 },
    ///0x0nnn
    JumpToSystemAddress { address: u16 },
*/

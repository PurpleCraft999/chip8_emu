use crate::Chip8Emulator;

#[test]
fn single_byte_load() {
    let mut chip8 = Chip8Emulator::new();

    chip8.load_bytes_into_memory(&[0x48]);

    let mut correct_bytes = [0; 4096];
    correct_bytes[0x200] = 0x48;
    assert_eq!(correct_bytes, chip8.memory)
}
#[test]
fn two_byte_load() {
    let mut chip8 = Chip8Emulator::new();

    chip8.load_bytes_into_memory(&[0x53, 0x94]);
    let mut correct_bytes = [0; 4096];
    correct_bytes[512] = 0x53;
    correct_bytes[513] = 0x94;
    assert_eq!(correct_bytes, chip8.memory)
}
#[test]
fn clear_screen() {
    let mut chip8 = Chip8Emulator::new();
    //loads the clear screen command
    chip8.load_bytes_into_memory(&[0x00, 0xE0]);
    //set random areas to 1 to see if they get cleared
    chip8.display[1032] = 1;
    chip8.display[2020] = 1;
    chip8.cycle();
    assert_eq!([0; 2048], chip8.display)
}
#[test]
fn set_v_register() {
    let mut chip8 = Chip8Emulator::new();
    //set reg 1 to 0x08
    chip8.load_bytes_into_memory(&[0x61, 0x08]);
    chip8.cycle();
    assert_eq!(0x08, chip8.get_v(1))
}
#[test]
fn load_index_registry() {
    let mut chip8 = Chip8Emulator::new();
    //set i reg to 0x250
    chip8.load_bytes_into_memory(&[0xA2, 0x50]);
    chip8.cycle();
    assert_eq!(0x250, chip8.index_register)
}
#[test]
fn jump() {
    let mut chip8 = Chip8Emulator::new();
    //jump to memory 0xF60
    chip8.load_bytes_into_memory(&[0x1F, 0x60]);
    chip8.cycle();
    assert_eq!(0xF60, chip8.program_counter)
}
#[test]
fn add() {
    let mut chip8 = Chip8Emulator::new();
    //set reg 0 to 1 then add 0x80 add 0x8 to registry 0
    chip8.load_bytes_into_memory(&[0x60, 0x1, 0x70, 0x80]);

    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x81, chip8.get_v(0))
}
#[test]
fn display() {
    let mut chip8 = Chip8Emulator::new();
    //set i to 0x204(where sprite is) then draw at x=0 y=1 for 5 rows
    chip8.load_bytes_into_memory(&[0xA2, 0x04, 0xD0, 0x15, 0x88, 0x50, 0xF8, 0xA8, 0x70]);
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
    chip8.cycle();
    chip8.cycle();
    // chip8.cycle();
    assert_eq!(correct_display, chip8.display)
}

/*
NEEDS TESTS
Return,
*/

#[test]
fn skip_if_register_eq_value() {
    let mut chip8 = Chip8Emulator::new();
    //set v0 to 1 then check if it equals one
    chip8.load_bytes_into_memory(&[
        0x60, 0x01, 0x30, 0x01, //this should never run
        0x62, 0x05,
    ]);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}
#[test]
fn skip_if_register_not_eq_value() {
    let mut chip8 = Chip8Emulator::new();
    //set v0 to 1 the check if != to 0
    chip8.load_bytes_into_memory(&[
        0x60, 0x01, 0x40, 0x00, //this should never run
        0x62, 0x05,
    ]);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}
#[test]
fn skip_if_registers_eq() {
    let mut chip8 = Chip8Emulator::new();
    //set v0 and v1 to 1 then check if equal
    chip8.load_bytes_into_memory(&[
        0x60, 0x01, 0x61, 0x01, 0x50, 0x10, //this should never run
        0x62, 0x05,
    ]);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}
#[test]
fn skip_if_registers_not_eq() {
    let mut chip8 = Chip8Emulator::new();
    //set v0 and v1 to 1 then check if equal
    chip8.load_bytes_into_memory(&[
        0x60, 0x01, 0x61, 0x02, 0x90, 0x10, //this should never run
        0x62, 0x05,
    ]);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    //if it equals 5 then the reg was set and was not skiped
    assert_ne!(chip8.get_v(2), 5)
}
#[test]
fn set_register_eq() {
    let mut chip8 = Chip8Emulator::new();
    //set register 1 to 0x32 then set reg 0 the the value of reg 1
    chip8.load_bytes_into_memory(&[0x61, 0x32, 0x80, 0x10]);
    chip8.cycle();
    chip8.cycle();
    assert_eq!(chip8.get_v(0), chip8.get_v(1))
}
#[test]
fn call() {
    let mut chip8 = Chip8Emulator::new();
    //make a call to a mem value
    chip8.load_bytes_into_memory(&[0x22, 0x04, 0x0, 0x0, 0x60, 0x01]);
    chip8.cycle();
    chip8.cycle();
    assert_eq!(chip8.get_v(0), 1);
    assert_eq!(chip8.stack_pointer, 1);
    assert_eq!(chip8.program_counter, 0x206)
}
#[test]
fn return_opcode_test() {
    let mut chip8 = Chip8Emulator::new();
    //does a call to 0x204 then sets reg 1 to 5 then returns and sets reg 0 to 1
    chip8.load_bytes_into_memory(&[0x22, 0x04, 0x60, 0x01, 0x61, 0x05, 0x00, 0xEE]);
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    chip8.cycle();
    assert_eq!(chip8.get_v(1), 5);
    assert_eq!(chip8.get_v(0), 1);
}
#[test]
fn test_or_logic() {
    let mut chip8 = Chip8Emulator::new();
    // set reg 0 to 5, reg 1 to 0xA then preform bitwise or
    chip8.load_bytes_into_memory(&[0x60, 0x05, 0x61, 0x0A, 0x80, 0x11]);

    chip8.cycle();
    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x0F, chip8.get_v(0));
}

#[test]
fn test_and_logic() {
    let mut chip8 = Chip8Emulator::new();
    //loads reg 0 with 0xC and reg 1 with 0x06 then preforms bit and on them
    chip8.load_bytes_into_memory(&[0x60, 0x0C, 0x61, 0x06, 0x80, 0x12]);

    chip8.cycle();
    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x04, chip8.get_v(0));
}

#[test]
fn test_xor_logic() {
    let mut chip8 = Chip8Emulator::new();
    // set reg 0 to 0xFF the reg 1 to 0xAA the preforms bit xor
    chip8.load_bytes_into_memory(&[0x60, 0xFF, 0x61, 0xAA, 0x80, 0x13]);

    chip8.cycle();
    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x55, chip8.get_v(0));
}
///ai
#[test]
fn test_add_reg_8xy4() {
    let mut chip8 = Chip8Emulator::new();
    // 60FE -> V0 = 254 (0xFE)
    // 6103 -> V1 = 3   (0x03)
    // 8014 -> V0 = V0 + V1 (Should result in 257, so V0=1 and VF=1)
    chip8.load_bytes_into_memory(&[0x60, 0xFE, 0x61, 0x03, 0x80, 0x14]);

    for _ in 0..3 {
        chip8.cycle();
    }

    assert_eq!(0x01, chip8.get_v(0)); // 254 + 3 = 257 (wraps to 1)
    assert_eq!(1, chip8.get_v(0xF)); // Carry flag should be set
}
///ai
#[test]
fn test_sub_reg_8xy5() {
    let mut chip8 = Chip8Emulator::new();
    // 600A -> V0 = 10
    // 6105 -> V1 = 5
    // 8015 -> V0 = V0 - V1 (10 - 5 = 5, VF=1 because NO borrow)
    chip8.load_bytes_into_memory(&[0x60, 0x0A, 0x61, 0x05, 0x80, 0x15]);

    for _ in 0..3 {
        chip8.cycle();
    }

    assert_eq!(0x05, chip8.get_v(0));
    assert_eq!(1, chip8.get_v(0xF)); // VF=1 means no borrow (result > 0)
}
///ai
#[test]
fn test_sub_reg_yx_8xy7() {
    let mut chip8 = Chip8Emulator::new();
    // 6005 -> V0 = 5
    // 610A -> V1 = 10
    // 8017 -> V0 = V1 - V0 (10 - 5 = 5, VF=1 because NO borrow)
    chip8.load_bytes_into_memory(&[0x60, 0x05, 0x61, 0x0A, 0x80, 0x17]);

    for _ in 0..3 {
        chip8.cycle();
    }

    assert_eq!(0x05, chip8.get_v(0));
    assert_eq!(1, chip8.get_v(0xF));
}
///ai
#[test]
fn test_shr_8xy6() {
    let mut chip8 = Chip8Emulator::new();
    // 6003 -> V0 = 3 (0000 0011)
    // 8006 -> V0 = V0 >> 1 (Shift right)
    chip8.load_bytes_into_memory(&[0x60, 0x03, 0x80, 0x06]);

    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x01, chip8.get_v(0)); // 3 >> 1 = 1
    assert_eq!(1, chip8.get_v(0xF)); // LSB was 1, so VF = 1
}
///ai
#[test]
fn test_shl_8xye() {
    let mut chip8 = Chip8Emulator::new();
    // 6080 -> V0 = 128 (1000 0000)
    // 800E -> V0 = V0 << 1 (Shift left)
    chip8.load_bytes_into_memory(&[0x60, 0x80, 0x80, 0x0E]);

    chip8.cycle();
    chip8.cycle();

    assert_eq!(0x00, chip8.get_v(0)); // 128 << 1 = 256 (wraps to 0)
    assert_eq!(1, chip8.get_v(0xF)); // MSB was 1, so VF = 1
}

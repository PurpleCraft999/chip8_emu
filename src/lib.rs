#![allow(dead_code)]
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
#[repr(u16)]
pub enum Chip8OpCode {
    ///0x00E0
    ClearScreen,
    ///0x06        
    LoadVRegistry {
        registry: u8,
        value: u8,
    },
    ///0xA
    LoadIndexRegistry(u16),
    ///0xD
    Display {
        x: u8,
        y: u8,
        n: u8,
    },
    ///0x1
    Jump {
        address: u16,
    },
    ///0x7
    Add {
        registry: u8,
        value: u8,
    },
    ///0x3
    SkipIfEqValue {
        registry: u8,
        value: u8,
    },
    ///0x4
    SkipIfNotEqValue {
        registry: u8,
        value: u8,
    },
    ///0x5xy0
    SkipIfRegEq {
        registry: u8,
        registry2: u8,
    },
    ///0x9xy0
    SkipIfRegNotEq {
        registry: u8,
        registry2: u8,
    },
    ///0x2
    Call {
        address: u16,
    },
    ///0x00EE
    Return,
    ///8ny0
    SetRegEq {
        registry: u8,
        registry2: u8,
    },

    ///8xy1
    BitOr {
        registry: u8,
        registry2: u8,
    },
    BitAnd {
        registry: u8,
        registry2: u8,
    },
    BitXor {
        registry: u8,
        registry2: u8,
    },
    //below here needs tests
    AddReg {
        registry: u8,
        registry2: u8,
    },
    SubReg {
        registry: u8,
        registry2: u8,
    },
    ShiftRight {
        registry: u8,
        registry2: u8,
    },
    SubRegYX {
        registry: u8,
        registry2: u8,
    },
    ShiftLeft {
        registry: u8,
        registry2: u8,
    },
}
impl Chip8OpCode {
    fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr> {
        //gets the any memory address if it has one
        // nnn
        let address = opcode & 0x0FFF;
        //gets the registry number
        let registry = ((opcode & 0x0F00) >> 8) as u8;
        //gets the value
        // kk
        let value = (opcode & 0x00FF) as u8;

        let registry2 = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;

        //get just the operator type
        match (opcode & 0xF000) >> 12 {
            //op type is 0 it must be one of these
            0x0 => match opcode {
                0xE0 => Ok(Self::ClearScreen),
                0xEE => Ok(Self::Return),
                _ => Err(UnkownOpCodeErr(opcode)),
            },
            0x6 => Ok(Self::LoadVRegistry { registry, value }),
            0xA => Ok(Self::LoadIndexRegistry(address)),
            0xD => Ok(Self::Display {
                x: registry,
                y: registry2,
                n,
            }),
            0x1 => Ok(Self::Jump { address }),
            0x7 => Ok(Self::Add { registry, value }),
            0x3 => Ok(Self::SkipIfEqValue { registry, value }),
            0x4 => Ok(Self::SkipIfNotEqValue { registry, value }),
            0x5 if n == 0 => Ok(Self::SkipIfRegEq {
                registry,
                registry2,
            }),
            0x9 if n == 0 => Ok(Self::SkipIfRegNotEq {
                registry,
                registry2,
            }),
            0x2 => Ok(Self::Call { address }),
            0x8 if n == 0 => Ok(Self::SetRegEq {
                registry,
                registry2,
            }),
            0x8 if n == 1 => Ok(Self::BitOr {
                registry,
                registry2,
            }),
            0x8 if n == 2 => Ok(Self::BitAnd {
                registry,
                registry2,
            }),
            0x8 if n == 3 => Ok(Self::BitXor {
                registry,
                registry2,
            }),
            0x8 if n == 4 => Ok(Self::AddReg {
                registry,
                registry2,
            }),
            0x8 if n == 5 => Ok(Self::SubReg {
                registry,
                registry2,
            }),
            0x8 if n == 6 => Ok(Self::ShiftRight {
                registry,
                registry2,
            }),
            0x8 if n == 7 => Ok(Self::SubRegYX {
                registry,
                registry2,
            }),
            0x8 if n == 0xE => Ok(Self::ShiftLeft {
                registry,
                registry2,
            }),
            _ => Err(UnkownOpCodeErr(opcode)),
        }
    }
}

struct UnkownOpCodeErr(u16);

const FONT_SET: [u8; 80] = [
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
pub const SCREEN_WIDTH: u8 = 64;
pub const SCREEN_HEIGHT: u8 = 32;
pub struct Chip8Emulator {
    memory: [u8; 4096],
    //usize to index memory
    program_counter: usize,
    v_registers: [u8; 16],
    stack: [usize; 16],
    stack_pointer: usize,
    ///used to store 16bit memory addresses
    index_register: u16,
    ///64 by 32 screen with black or white pixels
    display: [u8; 2048],
    delay_timer: u8,
    sound_timer: u8,
    draw_flag: bool,
}
impl Chip8Emulator {
    ///makes a completely blank chip8 emulator
    pub const fn new() -> Self {
        Self {
            //all the memory the emulator needs
            memory: [0; 4096],
            //the program counter
            program_counter: 0,
            // the variable registers
            v_registers: [0; 16],
            stack: [0; 16],
            stack_pointer: 0,
            index_register: 0,
            //the graphics buffer
            display: [0; 2048],
            sound_timer: 0,
            delay_timer: 0,
            draw_flag: false,
        }
    }
    pub fn get_draw_flag(&self) -> bool {
        self.draw_flag
    }
    pub fn draw_flag_off(&mut self) {
        self.draw_flag = false;
    }
    pub fn get_display(&self) -> &[u8; 2048] {
        &self.display
    }
    ///makes a chip8 emulator with the necisary items loaded into memory
    pub const fn init() -> Self {
        let mut chip8 = Self::new();
        let mut i = 0;
        //done this way due to const
        while i < FONT_SET.len() {
            chip8.memory[i] = FONT_SET[i];
            i += 1;
        }

        chip8
    }
    pub fn load_game(&mut self, file_path: &Path) -> io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut game_bytes = Vec::new();
        file.read_to_end(&mut game_bytes)?;
        self.load_bytes_into_memory(&game_bytes);
        Ok(())
    }
    fn load_bytes_into_memory(&mut self, bytes: &[u8]) {
        for (i, byte) in bytes.iter().enumerate() {
            //0x200 is the start of the useable memory
            self.memory[i + 0x200] = *byte;
        }
        //the starting location for game memory
        //decimal 512
        self.program_counter = 0x200;
    }

    pub fn cycle(&mut self) {
        //ensures the program counter is always even
        if !self.program_counter.is_multiple_of(2) {
            panic!(
                "program_counter is misaligned. value:{}",
                self.program_counter
            )
        }
        //this is for the few opcodes that dont want the program counter to increase normally
        let mut increase_program_counter = true;

        //gets the opcode from the next two bytes
        let opcode = u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]);
        // println!("opcode: {opcode:X}");

        let opcode = match Chip8OpCode::decode(opcode) {
            Ok(opcode) => opcode,
            //if zero just increase program counter then return
            Err(UnkownOpCodeErr(0)) => {
                self.increase_program_counter();
                return;
            }

            Err(UnkownOpCodeErr(e)) => {
                println!("unkown opcode:{e:X}");
                self.increase_program_counter();
                return;
            }
        };

        match opcode {
            Chip8OpCode::ClearScreen => {
                self.display = [0; 2048];
                self.draw_flag = true
            }
            Chip8OpCode::LoadVRegistry { registry, value } => self.set_v(registry, value),
            Chip8OpCode::LoadIndexRegistry(value) => self.index_register = value,
            Chip8OpCode::Display { x, y, n } => {
                let vx = self.get_v(x) % SCREEN_WIDTH;
                let vy = self.get_v(y) % SCREEN_HEIGHT;
                //resets the collision register
                self.v_registers[0xF] = 0;

                for row in 0..n {
                    //read the pixel from memory
                    let pixel = self.get_memory(self.index_register + row as u16);
                    //for each bit in the byte
                    for col in 0..8 {
                        if pixel & (0x80 >> col) != 0 {
                            let x_coord = ((vx + col) % SCREEN_WIDTH) as usize;
                            let y_coord = ((vy + row) % SCREEN_HEIGHT) as usize;

                            let idx = x_coord + (y_coord * SCREEN_WIDTH as usize);
                            //this is a collision
                            if self.display[idx] == 1 {
                                self.set_v(0xF, 1);
                            }
                            self.display[idx] ^= 1;
                        }
                    }
                }
                self.draw_flag = true;
            }
            Chip8OpCode::Jump { address: addr } => {
                self.program_counter = addr as usize;
                increase_program_counter = false
            }
            Chip8OpCode::Add { registry, value } => {
                self.set_v(registry, self.get_v(registry).wrapping_add(value))
            }
            Chip8OpCode::SkipIfEqValue { registry, value } => {
                if self.get_v(registry) == value {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::SkipIfNotEqValue { registry, value } => {
                if self.get_v(registry) != value {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::SkipIfRegEq {
                registry,
                registry2,
            } => {
                if self.get_v(registry) == self.get_v(registry2) {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::SkipIfRegNotEq {
                registry,
                registry2,
            } => {
                if self.get_v(registry) != self.get_v(registry2) {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::Call { address } => {
                self.stack[self.stack_pointer] = self.program_counter + 2;
                self.stack_pointer += 1;
                self.program_counter = address as usize;
                increase_program_counter = false
            }
            Chip8OpCode::Return => {
                self.stack_pointer -= 1;
                self.program_counter = self.stack[self.stack_pointer];
                self.stack[self.stack_pointer] = 0;
                increase_program_counter = false;
            }
            Chip8OpCode::SetRegEq {
                registry,
                registry2,
            } => self.set_v(registry, self.get_v(registry2)),
            Chip8OpCode::BitOr {
                registry,
                registry2,
            } => self.set_v(registry, self.get_v(registry) | self.get_v(registry2)),
            Chip8OpCode::BitAnd {
                registry,
                registry2,
            } => self.set_v(registry, self.get_v(registry) & self.get_v(registry2)),
            Chip8OpCode::BitXor {
                registry,
                registry2,
            } => self.set_v(registry, self.get_v(registry) ^ self.get_v(registry2)),
            Chip8OpCode::AddReg {
                registry,
                registry2,
            } => {
                let add = self.get_v(registry).overflowing_add(self.get_v(registry2));

                self.set_v(registry, add.0);
                self.set_v(0xF, add.1 as u8)
            }
            Chip8OpCode::SubReg {
                registry,
                registry2,
            } => {
                let sub = self.get_v(registry).overflowing_sub(self.get_v(registry2));

                self.set_v(registry, sub.0);
                self.set_v(0xF, !sub.1 as u8)
            }
            Chip8OpCode::ShiftRight { registry, .. } => {
                let least = self.get_v(registry) & 1;
                let shr = self.get_v(registry).wrapping_shr(1);

                self.set_v(registry, shr);
                if least == 1 {
                    self.set_v(0xF, 1)
                } else {
                    self.set_v(0xF, 0)
                }
            }
            Chip8OpCode::ShiftLeft { registry, .. } => {
                let most = self.get_v(registry) & 0x80;

                let shl = self.get_v(registry).wrapping_shl(1);

                self.set_v(registry, shl);
                if most == 128 {
                    self.set_v(0xF, 1)
                } else {
                    self.set_v(0xF, 0)
                }
            }
            Chip8OpCode::SubRegYX {
                registry,
                registry2,
            } => {
                let sub = self.get_v(registry2).overflowing_sub(self.get_v(registry));

                self.set_v(registry, sub.0);
                self.set_v(0xF, !sub.1 as u8)
            }
        }

        if increase_program_counter {
            self.increase_program_counter()
        }
    }
    fn increase_program_counter(&mut self) {
        self.program_counter += 2
    }
    ///gets from the v registry
    fn get_v(&self, addr: u8) -> u8 {
        self.v_registers[addr as usize]
    }
    fn get_memory(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    fn set_v(&mut self, addr: u8, value: u8) {
        self.v_registers[addr as usize] = value
    }
}

#[cfg(test)]
mod test {
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
}

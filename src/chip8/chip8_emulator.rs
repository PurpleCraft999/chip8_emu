use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    time::UNIX_EPOCH,
};

use crate::{
    Chip8OpCode, SCREEN_HEIGHT, SCREEN_WIDTH, UnkownOpCodeErr,
    chip8::chip8_emulator_helpers::Chip8Audio,
};

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

/*cave explorer
    5=w
    7=a
    8=s
    9=d
*/

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
    audio_player: Chip8Audio,
    ///holds if that key is pressed or not
    keys: [bool; 16],
    //below here is my stuff and is not strictly neccesary
    ///only used for WaitForKey (Fx0A)
    active_key: Option<u8>,
    ///has something been loaded into memory
    has_memory_loaded: bool,
}
impl Chip8Emulator {
    ///makes a completely blank chip8 emulator
    pub fn new() -> Self {
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
            keys: [false; 16],
            audio_player: Chip8Audio::new(),
            active_key: None,
            has_memory_loaded: false,
        }
    }
    pub fn load_font(&mut self) {
        for (i, byte) in FONT_SET.iter().enumerate() {
            self.memory[i] = *byte
        }
    }
    ///makes a chip8 emulator with the necisary items loaded into memory
    pub fn init() -> Self {
        let mut chip8 = Self::new();
        chip8.load_font();

        chip8
    }
    pub fn load_game(&mut self, file_path: &Path) -> io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut game_bytes = Vec::new();
        file.read_to_end(&mut game_bytes)?;
        self.load_bytes_into_memory(&game_bytes);
        Ok(())
    }
    pub fn load_bytes_into_memory(&mut self, bytes: &[u8]) {
        // println!("byte len:{}", bytes.len());
        assert!(bytes.len() <= self.memory.len() - 512);

        self.reset();

        for (i, byte) in bytes.iter().enumerate() {
            //0x200 is the start of the useable memory
            self.memory[i + 0x200] = *byte;
        }
        //the starting location for game memory
        //decimal 512
        self.program_counter = 0x200;
        self.has_memory_loaded = true;
    }

    pub fn reset(&mut self) {
        *self = Chip8Emulator::new();
    }

    pub fn tick_timers(&mut self) {
        if !self.has_memory_loaded {
            return;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        } else {
            self.audio_player.pause();
        }
    }
    ///if it comes a cross empty memory it will return `Chip8OpCode::Add {registry: 0,value: 0}`
    pub fn current_op_code(&self) -> Chip8OpCode {
        //gets the opcode from the next two bytes
        let opcode = u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]);
        // println!("opcode: {opcode:X}");

        match Chip8OpCode::decode(opcode) {
            Ok(opcode) => opcode,
            //run a useless opcode
            Err(UnkownOpCodeErr(0)) => Chip8OpCode::Add {
                registry: 0,
                value: 0,
            },

            Err(UnkownOpCodeErr(e)) => {
                println!("unkown opcode:{e:X}");
                Chip8OpCode::Add {
                    registry: 0,
                    value: 0,
                }
            }
        }
    }
    fn increase_program_counter(&mut self) {
        if !self.has_memory_loaded {
            return;
        }
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
        if !self.has_memory_loaded {
            return;
        }
        self.v_registers[addr as usize] = value
    }
    pub fn set_key(&mut self, index: usize, state: bool) {
        if !self.has_memory_loaded {
            return;
        }
        self.keys[index] = state
    }
    pub fn get_draw_flag(&self) -> bool {
        self.draw_flag
    }
    pub fn done_drawing(&mut self) {
        self.draw_flag = false;
    }
    pub fn get_display(&self) -> &[u8; 2048] {
        &self.display
    }
}

impl Default for Chip8Emulator {
    fn default() -> Self {
        Self::new()
    }
}

#[path = "../tests/chip8_emulator_opcode_tests.rs"]
#[cfg(test)]
mod chip8_emulator_opcode_tests;

impl Chip8Emulator {
    pub fn cycle(&mut self) {
        //nothing to run
        if !self.has_memory_loaded {
            return;
        }

        //this is for the few opcodes that dont want the program counter to increase normally
        let mut increase_program_counter = true;

        let opcode = self.current_op_code();

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
                            let x_coord = (vx + col) as usize;
                            let y_coord = (vy + row) as usize;
                            if x_coord < SCREEN_WIDTH as usize && y_coord < SCREEN_HEIGHT as usize {
                                let idx = x_coord + (y_coord * SCREEN_WIDTH as usize);
                                if idx < self.display.len() {
                                    //this is a collision
                                    if self.display[idx] == 1 {
                                        self.set_v(0xF, 1);
                                    }
                                    self.display[idx] ^= 1;
                                }
                            }
                        }
                    }
                }
                self.draw_flag = true;
            }
            Chip8OpCode::Jump { address } => {
                self.program_counter = address as usize;
                increase_program_counter = false
            }
            Chip8OpCode::JumpPlusV0 { address } => {
                self.program_counter = address as usize + self.get_v(0) as usize;
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
            } => {
                self.set_v(registry, self.get_v(registry) | self.get_v(registry2));
                self.set_v(0xF, 0);
            }
            Chip8OpCode::BitAnd {
                registry,
                registry2,
            } => {
                self.set_v(registry, self.get_v(registry) & self.get_v(registry2));
                self.set_v(0xF, 0);
            }
            Chip8OpCode::BitXor {
                registry,
                registry2,
            } => {
                self.set_v(registry, self.get_v(registry) ^ self.get_v(registry2));
                self.set_v(0xF, 0);
            }
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
            Chip8OpCode::ShiftRight {
                registry,
                registry2,
            } => {
                let reg_to_shift = registry2;
                let least = self.get_v(reg_to_shift) & 1;
                let shr = self.get_v(reg_to_shift).wrapping_shr(1);

                self.set_v(registry, shr);
                if least == 1 {
                    self.set_v(0xF, 1)
                } else {
                    self.set_v(0xF, 0)
                }
            }
            Chip8OpCode::ShiftLeft {
                registry,
                registry2,
            } => {
                let reg_to_shift = registry2;

                let most = self.get_v(reg_to_shift) & 0x80;

                let shl = self.get_v(reg_to_shift).wrapping_shl(1);

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
            Chip8OpCode::LoadMemIntoRegs { max_registry } => {
                for i in 0..=max_registry {
                    let mem = self.get_memory(self.index_register + i as u16);
                    self.set_v(i, mem);
                }
                self.index_register += 1 + max_registry as u16;
            }
            Chip8OpCode::StoreRegsIntoMem { max_registry } => {
                for i in 0..=max_registry {
                    let v = self.get_v(i);
                    self.memory[self.index_register as usize + i as usize] = v;
                }
                self.index_register += 1 + max_registry as u16;
            }
            Chip8OpCode::StoreVXAsBinary { registry } => {
                let n = self.get_v(registry);
                let hundreds = n / 100;
                let tens = (n / 10) % 10;
                let ones = n % 10;
                let index = self.index_register as usize;
                self.memory[index] = hundreds;
                self.memory[index + 1] = tens;
                self.memory[index + 2] = ones;
            }
            Chip8OpCode::AddAssignI { registry } => {
                self.index_register += self.get_v(registry) as u16
            }
            Chip8OpCode::SkipIfNotPressed { registry } => {
                if !self.keys[self.get_v(registry) as usize] {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::SkipIfPressed { registry } => {
                if self.keys[self.get_v(registry) as usize] {
                    self.increase_program_counter();
                }
            }
            Chip8OpCode::PutDelayInRegX { registry } => self.set_v(registry, self.delay_timer),
            Chip8OpCode::SetDelayTimer { registry: time } => self.delay_timer = self.get_v(time),
            Chip8OpCode::WaitForKey { registry } => {
                if self.active_key.is_none() {
                    for (i, &is_down) in self.keys.iter().enumerate() {
                        if is_down {
                            self.active_key = Some(i as u8);
                            break;
                        }
                    }
                    increase_program_counter = false
                } else if let Some(key_idx) = self.active_key {
                    if !self.keys[key_idx as usize] {
                        self.set_v(registry, key_idx);
                        self.active_key = None;
                    } else {
                        increase_program_counter = false
                    }
                }
            }
            Chip8OpCode::Random { registry, value } => self.set_v(
                registry,
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u8
                    & value,
            ),
            Chip8OpCode::SetSoundTimer { registry: time } => {
                self.sound_timer = self.get_v(time);
                self.audio_player.play();
            }
            Chip8OpCode::GetFontStart { registry } => {
                //load font if not loaded
                if self.memory[0] != 0xF0 {
                    self.load_font();
                }
                self.index_register = registry as u16 * 5;
            }
            Chip8OpCode::JumpToSystemAddress { .. } => println!("unsuported operation 0x0nnn"),
        }

        if increase_program_counter {
            self.increase_program_counter()
        }
    }
}

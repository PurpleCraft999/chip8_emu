#![allow(dead_code)]
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};
#[repr(u16)]
pub enum Chip8OpCode {
    ///E0
    ClearScreen,
    ///6        
    LoadVRegistry{registry:u8, value:u8},
    ///A
    LoadIndexRegistry(u16),
    ///hex value:D
    ///
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I.
    /// These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    /// Sprites are XORed onto the existing screen. If this causes any pixels to be erased,
    /// VF is set to 1, otherwise it is set to 0.
    /// If the sprite is positioned so part of it is outside the coordinates of the display,
    /// it wraps around to the opposite side of the screen
    Display { x: u8, y: u8, n: u8 },
    Jump{addr:u16}
}
impl Chip8OpCode {
    fn decode(opcode: u16) -> Self {
        //gets the any memory address if it has one
        // nnn
        let address = opcode & 0x0FFF;
        //gets the registry number
        let x = ((opcode & 0x0F00) >> 8) as u8;
        //gets the value
        // kk
        let value = (opcode & 0x00FF) as u8;

        let y = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;

        //get just the operator type
        match (opcode & 0xF000) >> 12 {
            //op type is 0 it must be one of these
            0x0 => match opcode {
                0xE0 => Self::ClearScreen,
                op => panic!("unkown opcode {op:X}"),
            },
            0x6 => Self::LoadVRegistry{registry:x, value},
            0xA => Self::LoadIndexRegistry(address),
            0xD => Self::Display { x, y, n },
            0x1 => Self::Jump { addr: address },
            op => panic!("unkown opcode {op:X}"),
        }
    }
}

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
    stack: [u8; 16],
    stack_pointer: u8,
    ///used to store 16bit memory addresses
    index_register: u16,
    ///64 by 32 screen with black or white pixels
    pub graphics: [u8; 2048],
    delay_timer: u8,
    sound_timer: u8,
    display_flag: bool,
}
impl Chip8Emulator {
    ///makes a completly blank chip8 emulator
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
            graphics: [0; 2048],
            sound_timer: 0,
            delay_timer: 0,
            display_flag: false,
        }
    }
    pub fn display_flag(&self) -> bool {
        self.display_flag
    }
    ///makes a chip8 emulator with the necisary items loaded into memory
    pub fn init() -> Self {
        let mut chip8 = Self::new();
        for (i, byte) in FONT_SET.iter().enumerate() {
            chip8.memory[i] = *byte;
        }
        //the starting location for game memory
        chip8.program_counter = 0x200;
        chip8
    }
    pub fn load_game(&mut self, file_path: &Path) -> io::Result<()> {
        let mut file = File::open(file_path)?;
        let mut game_bytes = Vec::new();
        file.read_to_end(&mut game_bytes)?;
        for (i, byte) in game_bytes.iter().enumerate() {
            //512 is the start of the games memory
            self.memory[i + 512] = *byte;
        }
        Ok(())
    }

    pub fn cycle(&mut self) {
        //ensures the program counter is always even
        if !self.program_counter.is_multiple_of(2) {
            panic!("program_counter is misaligned")
        }
        //this is for the few opcodes that dont want the program counter to increase normally
        let mut increase_program_counter = true;

        //gets the opcode from the next two bytes
        let opcode = u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]);
        println!("opcode: {opcode:X}");
        match Chip8OpCode::decode(opcode) {
            Chip8OpCode::ClearScreen => self.graphics = [0; 2048],
            Chip8OpCode::LoadVRegistry{registry, value} => self.set_v(registry, value),
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
                            if self.graphics[idx] == 1 {
                                self.set_v(0xF, 1);
                            }
                            self.graphics[idx] ^= 1;
                        }
                    }
                }
                self.display_flag = true;
            },
            Chip8OpCode::Jump { addr } => {self.program_counter=addr as usize;increase_program_counter=false}
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

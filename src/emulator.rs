pub trait Chip {
    type OpcodeType: Opcode;
    fn get_display(&self) -> &[u8];
    fn get_display_mut(&mut self) -> &mut [u8];
}

// impl<O: Opcode> Chip for Box<dyn Chip<OpcodeType = O>> {
//     type OpcodeType = O;
//     fn get_display(&self) -> &[u8] {
//         self.as_ref().get_display()
//     }
//     fn get_display_mut(&mut self) -> &mut [u8] {
//         self.as_mut().get_display_mut()
//     }
// }

pub trait Opcode: Sized {
    fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr>;
    fn execute_opcode<O: Opcode + 'static>(self, emu: &mut ChipEmulator<O>) -> bool;
    fn useless_opcode() -> Self;
}

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::{ UnkownOpCodeErr, chip8::chip8_emulator::Chip8Audio};

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

pub struct ChipEmulator<O: Opcode> {
    memory: [u8; 4096],
    //usize to index memory
    program_counter: usize,
    v_registers: [u8; 16],
    stack: [usize; 16],
    stack_pointer: usize,
    ///used to store 16bit memory addresses
    index_register: u16,
    // ///64 by 32 screen with black or white pixels
    // display: [u8; 2048],
    chip: Box<dyn Chip<OpcodeType = O>>,
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
impl<O: Opcode + 'static> ChipEmulator<O> {
    ///makes a completely blank chip8 emulator
    pub fn new<C: Chip<OpcodeType = O> + 'static>(chip: C) -> Self {
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
            chip: Box::new(chip),
            // display: [0; 2048],
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
    pub fn init<C: Chip<OpcodeType = O> + 'static>(chip: C) -> Self {
        let mut chip8 = Self::new(chip);
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
        // *self = ChipEmulator::new(self.chip);
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
    pub fn current_op_code(&self) -> O {
        //gets the opcode from the next two bytes
        let opcode = u16::from_be_bytes([
            self.memory[self.program_counter],
            self.memory[self.program_counter + 1],
        ]);
        // println!("opcode: {opcode:X}");

        match O::decode(opcode) {
            Ok(opcode) => opcode,
            //run a useless opcode
            Err(UnkownOpCodeErr(0)) => O::useless_opcode(),

            Err(UnkownOpCodeErr(e)) => {
                println!("unkown opcode:{e:X}");
                O::useless_opcode()
            }
        }
    }
    pub fn increase_program_counter(&mut self) {
        if !self.has_memory_loaded {
            return;
        }
        self.program_counter += 2
    }
    ///gets from the v registry
    pub fn get_v(&self, addr: u8) -> u8 {
        self.v_registers[addr as usize]
    }
    pub fn get_memory(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }
    pub fn set_v(&mut self, addr: u8, value: u8) {
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
    pub fn set_draw_flag(&mut self, flag: bool) {
        self.draw_flag = flag;
    }
    pub fn get_display(&self) -> &[u8] {
        self.chip.get_display()
    }
    pub fn get_display_mut(&mut self) -> &mut [u8] {
        self.chip.get_display_mut()
    }
    pub fn set_index_register(&mut self, addr: u16) {
        self.index_register = addr;
    }
    pub fn get_index_register(&self) -> u16 {
        self.index_register
    }
    pub fn set_program_counter(&mut self, addr: usize) {
        self.program_counter = addr;
    }
    pub fn get_program_counter(&self) -> usize {
        self.program_counter
    }
    pub fn cycle(&mut self) {
        //nothing to run
        if !self.has_memory_loaded {
            return;
        }

        let opcode = self.current_op_code();

        if opcode.execute_opcode(self) {
            self.increase_program_counter()
        }
    }
    pub fn push_stack(&mut self, item: usize) {
        self.stack[self.stack_pointer] = item;
        self.stack_pointer += 1;
    }
    pub fn pop_stack(&mut self) -> usize {
        self.stack_pointer -= 1;
        let value = self.stack[self.stack_pointer];
        self.stack[self.stack_pointer] = 0;
        value
    }
    pub fn set_memory(&mut self, addr: usize, value: u8) {
        self.memory[addr] = value;
    }
    pub fn get_key(&self, index: usize) -> bool {
        self.keys[index]
    }
    pub fn get_keys(&self) -> [bool; 16] {
        self.keys
    }
    pub fn set_sound_timer(&mut self, time: u8) {
        self.sound_timer = time;
    }
    pub fn play_sound(&self) {
        self.audio_player.play();
    }
    pub fn get_delay_time(&self) -> u8 {
        self.delay_timer
    }
    pub fn set_delay_timer(&mut self, time: u8) {
        self.delay_timer = time;
    }
    pub fn has_active_key(&self) -> bool {
        self.active_key.is_some()
    }
    pub fn get_active_key(&self) -> Option<u8> {
        self.active_key
    }
    pub fn set_active_key(&mut self, key: Option<u8>) {
        self.active_key = key;
    }
}

#[path = "tests/chip8_emulator_opcode_tests.rs"]
#[cfg(test)]
mod chip8_emulator_opcode_tests;

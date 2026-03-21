use std::time::UNIX_EPOCH;

use crate::{
    ChipEmulator,
    emulator::{Chip, Opcode},
};

pub enum Chip8Opcode {
    ///0x00E0
    ClearScreen,
    ///0x6        
    LoadVRegistry { registry: u8, value: u8 },
    ///0xA
    LoadIndexRegistry(u16),
    ///0xD
    Draw { x: u8, y: u8, n: u8 },
    ///0x1
    Jump { address: u16 },
    ///0x7
    Add { registry: u8, value: u8 },
    ///0x3
    SkipIfEqValue { registry: u8, value: u8 },
    ///0x4
    SkipIfNotEqValue { registry: u8, value: u8 },
    ///0x5xy0
    SkipIfRegEq { registry: u8, registry2: u8 },
    ///0x9xy0
    SkipIfRegNotEq { registry: u8, registry2: u8 },
    ///0x2
    Call { address: u16 },
    ///0x00EE
    Return,
    ///8ny0
    SetRegEq { registry: u8, registry2: u8 },

    ///8xy1
    BitOr { registry: u8, registry2: u8 },
    ///8xy2
    BitAnd { registry: u8, registry2: u8 },
    ///8xy3
    BitXor { registry: u8, registry2: u8 },
    ///8xy4
    AddReg { registry: u8, registry2: u8 },
    ///8xy5
    SubReg { registry: u8, registry2: u8 },
    ///8xy6
    ShiftRight { registry: u8, registry2: u8 },
    ///8xy7
    SubRegYX { registry: u8, registry2: u8 },
    ///8xyE
    ShiftLeft { registry: u8, registry2: u8 },
    /// Fx65
    ///loads from memory starting from i into registries including max_registry
    LoadMemIntoRegs { max_registry: u8 },
    ///Fx55
    ///stores the registries 0..=max_registry in memory starting from i
    StoreRegsIntoMem { max_registry: u8 },
    ///Fx33
    ///store binary-coded decimal representation of vX to memory at i, i + 1 and i + 2
    StoreVXAsBinary { registry: u8 },
    ///Fx1E
    AddAssignI { registry: u8 },
    ///ExA1
    SkipIfNotPressed { registry: u8 },
    ///Ex9E
    SkipIfPressed { registry: u8 },
    ///Fx07
    PutDelayInRegX { registry: u8 },
    ///Fx15
    ///sets the delay timer to the value of the register
    SetDelayTimer { registry: u8 },
    ///Fx18
    ///sets the sound timer to the value of the register
    SetSoundTimer { registry: u8 },
    ///Fx0A
    ///waits for a key press then stores it in the registey
    WaitForKey { registry: u8 },
    ///0xB
    JumpPlusV0 { address: u16 },
    ///0xC
    Random { registry: u8, value: u8 },
    ///Fx29
    GetFontStart { registry: u8 },
    ///0x0nnn
    JumpToSystemAddress { address: u16 },
}
impl Opcode for Chip8Opcode {
    fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr> {
        //gets the any memory address if it has one
        // nnn
        let address = opcode & 0xFFF;
        //gets the registry number
        let registry = ((opcode & 0xF00) >> 8) as u8;
        //gets the value
        // kk
        let value = (opcode & 0xFF) as u8;
        //y
        let registry2 = ((opcode & 0xF0) >> 4) as u8;
        let n = (opcode & 0xF) as u8;
        let operator_type = (opcode & 0xF000) >> 12;
        //get just the operator type
        match operator_type {
            //op type is 0 it must be one of these
            0x0 => match opcode {
                0xE0 => Ok(Self::ClearScreen),
                0xEE => Ok(Self::Return),
                //this is a blank address
                0 => Err(UnkownOpCodeErr(opcode)),
                _ => Ok(Self::JumpToSystemAddress { address }),
            },
            0x6 => Ok(Self::LoadVRegistry { registry, value }),
            0xA => Ok(Self::LoadIndexRegistry(address)),
            0xD => Ok(Self::Draw {
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
            0xF if registry2 == 6 && n == 5 => Ok(Self::LoadMemIntoRegs {
                max_registry: registry,
            }),
            0xF if registry2 == 5 && n == 5 => Ok(Self::StoreRegsIntoMem {
                max_registry: registry,
            }),
            0xF if registry2 == 3 && n == 3 => Ok(Self::StoreVXAsBinary { registry }),
            0xF if registry2 == 1 && n == 0xE => Ok(Self::AddAssignI { registry }),
            0xE if registry2 == 0xA && n == 1 => Ok(Self::SkipIfNotPressed { registry }),
            0xE if registry2 == 9 && n == 0xE => Ok(Self::SkipIfPressed { registry }),
            0xF if registry2 == 0 && n == 7 => Ok(Self::PutDelayInRegX { registry }),
            0xF if registry2 == 1 && n == 5 => Ok(Self::SetDelayTimer { registry }),
            0xF if registry2 == 0 && n == 0xA => Ok(Self::WaitForKey { registry }),
            0xF if registry2 == 1 && n == 8 => Ok(Self::SetSoundTimer { registry }),
            0xF if registry2 == 2 && n == 9 => Ok(Self::GetFontStart { registry }),
            0xB => Ok(Self::JumpPlusV0 { address }),
            0xC => Ok(Self::Random { registry, value }),
            _ => Err(UnkownOpCodeErr(opcode)),
        }
    }
    fn execute_opcode<C: Chip>(self, emu: &mut crate::emulator::ChipEmulator<C>) -> bool {
        //this is for the few opcodes that dont want the program counter to increase normally

        let mut increase_program_counter = true;
        match self {
            Chip8Opcode::ClearScreen => {
                emu.get_display_mut().fill(0);
                emu.set_draw_flag(true);
            }
            Chip8Opcode::LoadVRegistry { registry, value } => emu.set_v(registry, value),
            Chip8Opcode::LoadIndexRegistry(value) => emu.set_index_register(value),
            Chip8Opcode::Draw { x, y, n } => {
                opcode_draw(emu, x, y, n);
            }
            Chip8Opcode::Jump { address } => {
                emu.set_program_counter(address as usize);
                increase_program_counter = false
            }
            Chip8Opcode::JumpPlusV0 { address } => {
                emu.set_program_counter(address as usize + emu.get_v(0) as usize);
                increase_program_counter = false
            }
            Chip8Opcode::Add { registry, value } => {
                emu.set_v(registry, emu.get_v(registry).wrapping_add(value))
            }
            Chip8Opcode::SkipIfEqValue { registry, value } => {
                if emu.get_v(registry) == value {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::SkipIfNotEqValue { registry, value } => {
                if emu.get_v(registry) != value {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::SkipIfRegEq {
                registry,
                registry2,
            } => {
                if emu.get_v(registry) == emu.get_v(registry2) {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::SkipIfRegNotEq {
                registry,
                registry2,
            } => {
                if emu.get_v(registry) != emu.get_v(registry2) {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::Call { address } => {
                emu.push_stack(emu.get_program_counter() + 2);
                emu.set_program_counter(address as usize);
                increase_program_counter = false
            }
            Chip8Opcode::Return => {
                let value = emu.pop_stack();
                emu.set_program_counter(value);
                increase_program_counter = false;
            }
            Chip8Opcode::SetRegEq {
                registry,
                registry2,
            } => emu.set_v(registry, emu.get_v(registry2)),
            Chip8Opcode::BitOr {
                registry,
                registry2,
            } => {
                emu.set_v(registry, emu.get_v(registry) | emu.get_v(registry2));
                emu.set_v(0xF, 0);
            }
            Chip8Opcode::BitAnd {
                registry,
                registry2,
            } => {
                emu.set_v(registry, emu.get_v(registry) & emu.get_v(registry2));
                emu.set_v(0xF, 0);
            }
            Chip8Opcode::BitXor {
                registry,
                registry2,
            } => {
                emu.set_v(registry, emu.get_v(registry) ^ emu.get_v(registry2));
                emu.set_v(0xF, 0);
            }
            Chip8Opcode::AddReg {
                registry,
                registry2,
            } => {
                let add = emu.get_v(registry).overflowing_add(emu.get_v(registry2));

                emu.set_v(registry, add.0);
                emu.set_v(0xF, add.1 as u8)
            }
            Chip8Opcode::SubReg {
                registry,
                registry2,
            } => {
                let sub = emu.get_v(registry).overflowing_sub(emu.get_v(registry2));

                emu.set_v(registry, sub.0);
                emu.set_v(0xF, !sub.1 as u8)
            }
            Chip8Opcode::ShiftRight {
                registry,
                registry2,
            } => {
                opcode_rshift(emu, registry2, registry);
            }
            Chip8Opcode::ShiftLeft {
                registry,
                registry2,
            } => {
                opcode_lshift(emu, registry2, registry);
            }
            Chip8Opcode::SubRegYX {
                registry,
                registry2,
            } => {
                let sub = emu.get_v(registry2).overflowing_sub(emu.get_v(registry));

                emu.set_v(registry, sub.0);
                emu.set_v(0xF, !sub.1 as u8)
            }
            Chip8Opcode::LoadMemIntoRegs { max_registry } => {
                opcode_load_mem_into_reg(emu, max_registry, true);
            }
            Chip8Opcode::StoreRegsIntoMem { max_registry } => {
                opcode_store_reg_into_mem(emu, max_registry, true);
            }
            Chip8Opcode::StoreVXAsBinary { registry } => {
                let n = emu.get_v(registry);
                let hundreds = n / 100;
                let tens = (n / 10) % 10;
                let ones = n % 10;
                let index = emu.get_index_register() as usize;
                emu.set_memory(index, hundreds);
                emu.set_memory(index + 1, tens);
                emu.set_memory(index + 2, ones);
            }
            Chip8Opcode::AddAssignI { registry } => {
                emu.set_index_register(emu.get_index_register() + emu.get_v(registry) as u16);
            }
            Chip8Opcode::SkipIfNotPressed { registry } => {
                if !emu.get_key(emu.get_v(registry) as usize) {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::SkipIfPressed { registry } => {
                if emu.get_key(emu.get_v(registry) as usize) {
                    emu.increase_program_counter();
                }
            }
            Chip8Opcode::PutDelayInRegX { registry } => emu.set_v(registry, emu.get_delay_time()),
            Chip8Opcode::SetDelayTimer { registry: time } => emu.set_delay_timer(emu.get_v(time)),
            Chip8Opcode::WaitForKey { registry } => {
                if !emu.has_active_key() {
                    for (i, &is_down) in emu.get_keys().iter().enumerate() {
                        if is_down {
                            emu.set_active_key(Some(i as u8));
                            break;
                        }
                    }
                    increase_program_counter = false
                } else if let Some(key_idx) = emu.get_active_key() {
                    if !emu.get_key(key_idx as usize) {
                        emu.set_v(registry, key_idx);
                        emu.set_active_key(None);
                    } else {
                        increase_program_counter = false
                    }
                }
            }
            Chip8Opcode::Random { registry, value } => emu.set_v(
                registry,
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u8
                    & value,
            ),
            Chip8Opcode::SetSoundTimer { registry: time } => {
                emu.set_sound_timer(emu.get_v(time));
                emu.play_sound();
            }
            Chip8Opcode::GetFontStart { registry } => {
                //load font if not loaded
                if emu.get_memory(0) != 0xF0 {
                    emu.load_font_small();
                }
                emu.set_index_register(emu.get_v(registry) as u16 * 5);
            }
            Chip8Opcode::JumpToSystemAddress { address } => {
                println!("0x0nnn is unsupported {address:X}")
            }
        }
        increase_program_counter
    }
    fn useless_opcode() -> Self {
        Self::Add {
            registry: 0,
            value: 0,
        }
    }
}

pub fn opcode_rshift<C: Chip>(emu: &mut ChipEmulator<C>, shift_registry: u8, set_registry: u8) {
    let reg_to_shift = shift_registry;
    let least = emu.get_v(reg_to_shift) & 1;
    let shr = emu.get_v(reg_to_shift).wrapping_shr(1);

    emu.set_v(set_registry, shr);
    if least == 1 {
        emu.set_v(0xF, 1)
    } else {
        emu.set_v(0xF, 0)
    }
}
pub fn opcode_lshift<C: Chip>(emu: &mut ChipEmulator<C>, shift_registry: u8, set_registry: u8) {
    let reg_to_shift = shift_registry;

    let most = emu.get_v(reg_to_shift) & 0x80;

    let shl = emu.get_v(reg_to_shift).wrapping_shl(1);

    emu.set_v(set_registry, shl);
    if most == 128 {
        emu.set_v(0xF, 1)
    } else {
        emu.set_v(0xF, 0)
    }
}
pub fn opcode_load_mem_into_reg<C: Chip>(
    emu: &mut ChipEmulator<C>,
    max_registry: u8,
    increase_i: bool,
) {
    for i in 0..=max_registry {
        let mem = emu.get_memory(emu.get_index_register() + i as u16);
        emu.set_v(i, mem);
    }
    if increase_i {
        emu.set_index_register(emu.get_index_register() + 1 + max_registry as u16);
    }
}
pub fn opcode_store_reg_into_mem<C: Chip>(
    emu: &mut ChipEmulator<C>,
    max_registry: u8,
    increase_i: bool,
) {
    for i in 0..=max_registry {
        let v = emu.get_v(i);
        emu.set_memory(emu.get_index_register() as usize + i as usize, v);
    }
    if increase_i {
        emu.set_index_register(emu.get_index_register() + 1 + max_registry as u16);
    }
}
pub fn opcode_draw<C: Chip>(emu: &mut ChipEmulator<C>, x: u8, y: u8, n: u8) {
    let (width, height) = emu.get_display_size();

    let vx = emu.get_v(x) % width as u8;
    let vy = emu.get_v(y) % height as u8;
    emu.set_v(0xF, 0);

    for row in 0..n {
        //read the pixel from memory
        let pixel = emu.get_memory(emu.get_index_register() + row as u16);
        //for each bit in the byte
        for col in 0..8 {
            if pixel & (0x80 >> col) != 0 {
                let x_coord = (vx + col) as usize;
                let y_coord = (vy + row) as usize;
                if x_coord < width && y_coord < height {
                    let idx = x_coord + (y_coord * width);
                    if idx < emu.get_display().len() {
                        //this is a collision
                        if emu.get_display()[idx] == 1 {
                            emu.set_v(0xF, 1);
                        }
                        emu.get_display_mut()[idx] ^= 1;
                    }
                }
            }
        }
    }
    emu.set_draw_flag(true);
}

#[derive(Debug)]
pub struct UnkownOpCodeErr(pub(crate) u16);

use std::{fs::{self, File}, io::{self, Write}};

use crate::{
    Chip8Opcode, ChipEmulator, UnkownOpCodeErr, chip8::chip8_opcode::{
        opcode_load_mem_into_reg, opcode_lshift, opcode_rshift, opcode_store_reg_into_mem,
    }, emulator::{Chip, Opcode}
};

pub enum SuperChipOpcode {
    Chip8(Chip8Opcode),
    ///00FF
    RaiseScreenResolution,
    ///00FE
    LowerScreenResolution,
    ///DXY0
    LargeDraw {
        registry: u8,
        registry2: u8,
    },
    ///00FB
    ScrollRight,
    ///00CN
    ScrollDown {
        n: u8,
    },
    ///00FC
    ScrollLeft,
    ///FX30
    SetFontLarge {
        registry: u8,
    },
    ///00FD
    Exit,
    ///FX75
    SaveToStorage {
        registry: u8,
    },
    ///FX85
    LoadFromStorage {
        registry: u8,
    },
}
impl Opcode for SuperChipOpcode {
    fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr> {
        // let address = opcode & 0xFFF;
        //gets the registry number
        //x
        let registry = ((opcode & 0xF00) >> 8) as u8;
        // //gets the value
        // // kk
        // let value = (opcode & 0xFF) as u8;
        //y
        let registry2 = ((opcode & 0xF0) >> 4) as u8;
        let n = (opcode & 0xF) as u8;
        let operator_type = (opcode & 0xF000) >> 12;

        match operator_type {
            0x0 => match opcode {
                0xFF => Ok(Self::RaiseScreenResolution),
                0xFE => Ok(Self::LowerScreenResolution),
                0xFB => Ok(Self::ScrollRight),
                0xFC => Ok(Self::ScrollLeft),
                scroll_down if opcode >> 4 == 0xC => Ok(Self::ScrollDown { n }),
                _ => Ok(Self::Chip8(Chip8Opcode::decode(opcode)?)),
            },
            0xD if n == 0 => Ok(Self::LargeDraw {
                registry,
                registry2,
            }),
            0xF if registry2 == 0x7 && n == 0x5 => Ok(Self::SaveToStorage { registry }),
            0xF if registry2 == 0x8 && n == 0x5 => Ok(Self::LoadFromStorage { registry }),
            _ => Ok(Self::Chip8(Chip8Opcode::decode(opcode)?)),
        }

        // match Chip8Opcode::decode(opcode){
        //     Ok(opcode)=>Ok(Self::Chip8(opcode)),
        //     Err(_)=>{
        //         Err(crate::UnkownOpCodeErr(0))

        //     }
        // }
    }
    fn execute_opcode<C: Chip>(self, emu: &mut crate::ChipEmulator<C>) -> bool {
        let mut increase_program_counter = true;
        match self {
            Self::Chip8(opcode) => match opcode {
                Chip8Opcode::ShiftRight { registry, .. } => opcode_rshift(emu, registry, registry),
                Chip8Opcode::ShiftLeft { registry, .. } => opcode_lshift(emu, registry, registry),
                Chip8Opcode::StoreRegsIntoMem { max_registry } => {
                    opcode_store_reg_into_mem(emu, max_registry, false)
                }
                Chip8Opcode::LoadMemIntoRegs { max_registry } => {
                    opcode_load_mem_into_reg(emu, max_registry, false)
                }
                Chip8Opcode::BitOr {
                    registry,
                    registry2,
                } => emu.set_v(registry, emu.get_v(registry) | emu.get_v(registry2)),
                Chip8Opcode::BitAnd {
                    registry,
                    registry2,
                } => emu.set_v(registry, emu.get_v(registry) & emu.get_v(registry2)),
                Chip8Opcode::BitXor {
                    registry,
                    registry2,
                } => emu.set_v(registry, emu.get_v(registry) ^ emu.get_v(registry2)),
                Chip8Opcode::JumpPlusV0 { address } => {
                    emu.set_program_counter(
                        address as usize + emu.get_v(((address & 0xF00) >> 8) as u8) as usize,
                    );
                    increase_program_counter = false
                }
                _ => increase_program_counter = opcode.execute_opcode(emu),
            },

            Self::LowerScreenResolution => emu.resize_display(2048),
            Self::RaiseScreenResolution => emu.resize_display(8192),

            Self::ScrollRight => {
                let (width, _) = emu.get_display_size();
                // let width  = 64;
                for row in emu.get_display_mut().chunks_mut(width) {
                    // let len = row.len();

                    row.copy_within(0..width - 4, 4);
                    row[..4].fill(0);
                }
            }
            Self::ScrollLeft => {
                let (width, _) = emu.get_display_size();
                // let width  = 64;
                for row in emu.get_display_mut().chunks_mut(width) {
                    // let len = row.len();
                    row.reverse();
                    row.copy_within(0..width - 4, 4);
                    row[..4].fill(0);
                    row.reverse();
                }
            }
            Self::ScrollDown { n } => {
                let n = n as usize;
                let (width, height) = emu.get_display_size();
                let display = emu.get_display_mut();
                for y in (n..height).rev() {
                    let target_start = y * width;
                    let source_start = (y - n) * width;

                    display.copy_within(source_start..source_start + width, target_start);
                }

                display[0..n * width].fill(0);
            }
            Self::LargeDraw {
                registry,
                registry2,
            } => {
                emu.set_v(0xF, 0);
                let start_address = emu.get_index_register();
                let vx = emu.get_v(registry) as usize;
                let vy = emu.get_v(registry2) as usize;

                for dy in 0..16 {
                    let row_addr = start_address + (dy * 2) as u16;
                    let line = u16::from_be_bytes([
                        emu.get_memory(row_addr),
                        emu.get_memory(row_addr + 1),
                    ]);

                    for dx in 0..16 {
                        let px = (vx + dx) % 128;
                        let py = vy + dy;

                        if py >= 64 {
                            continue;
                        }

                        if (line & (0x8000 >> dx)) != 0 {
                            let loc = px + (py * 128);

                            if emu.get_display()[loc] == 1 {
                                emu.set_v(0xF, 1);
                            }

                            emu.get_display_mut()[loc] ^= 1;
                        }
                    }
                }
                emu.set_draw_flag(true);
            }
            Self::Exit => {
                emu.reset();
                emu.set_draw_flag(true);
            }
            Self::LoadFromStorage { registry } => if let Err(e) =load_from_storage(emu, registry){match e{
                StorageError::NoGameId=>println!("no program to save"),
                StorageError::RegistyTooHigh=>println!("tried to save to a registry that was too high"),
                StorageError::Io(io)=>println!("{io}"),
                StorageError::InvailidKey =>println!("could not parse the key in save")
            }},
            Self::SaveToStorage { registry } =>if let Err(e) =save_to_storage(emu, registry){match e{
                StorageError::NoGameId=>println!("no program to save"),
                StorageError::RegistyTooHigh=>println!("tried to save to a registry that was too high"),
                StorageError::Io(io)=>println!("{io}"),
                _=>()
            }},
            Self::SetFontLarge { registry } => {
                if emu.get_memory(81)==0{
                    emu.load_font_big();
                }
                emu.set_index_register((registry as u16 * 10)+81);
            },
        }
        increase_program_counter
    }
    fn useless_opcode() -> Self {
        Self::Chip8(Chip8Opcode::useless_opcode())
    }
}

fn save_to_storage<C:Chip>(emu:&mut ChipEmulator<C>,max_registry:u8)->Result<(), StorageError>{
    if let Some(id)=emu.get_game_id(){

    
        if max_registry>7{
            return Err(StorageError::RegistyTooHigh);
        }
        let mut save  = String::with_capacity(max_registry as usize+1);
        for registry in  0..=max_registry{

            save.push_str(&registry.to_string());
            save.push(':');
            save.push_str(&emu.get_v(registry).to_string());
            save.push('\n');
        }
        save.pop();
        
        let mut file = File::create(&format!("save/{id}.txt"))?;
        file.write_all(save.as_bytes())?;
        Ok(())
    } else{
        Err(StorageError::NoGameId)
    }

}

fn load_from_storage<C:Chip>(emu:&mut ChipEmulator<C>,max_registry:u8)->Result<(), StorageError>{
    if let Some(id)=emu.get_game_id(){
    if max_registry>7{
        return Err(StorageError::RegistyTooHigh);
    }
    let save_file = fs::read_to_string(&format!("save/{id}.txt"))?;
    for line in save_file.lines(){
        if let Some((key,value))= line.split_once(':'){
            let key = key.parse::<u8>().map_err(|_|StorageError::InvailidKey)?;
            if key > max_registry{
                continue;
            }
            let value = value.parse::<u8>().map_err(|_|StorageError::InvailidKey)?;
            emu.set_v(key, value);
        }
    }
    Ok(())
} else {
        Err(StorageError::NoGameId)
    }
}


pub enum StorageError{
    Io(io::Error),
    RegistyTooHigh,
    NoGameId,
    InvailidKey
}

impl From<io::Error> for StorageError{
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

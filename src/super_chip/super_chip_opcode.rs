use crate::{
    Chip8Opcode, UnkownOpCodeErr,
    chip8::chip8_opcode::{
        opcode_load_mem_into_reg, opcode_lshift, opcode_rshift, opcode_store_reg_into_mem,
    },
    emulator::Opcode,
};

pub enum SuperChipOpcode {
    Chip8(Chip8Opcode),
    RaiseScreenResolution,
    LowerScreenResolution,
    Draw{
        registry:u8,
        registry2:u8
    },
    ScrollRight,
    ScrollDown,
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
                0xC => Ok(Self::ScrollDown),
                _ => Ok(Self::Chip8(Chip8Opcode::decode(opcode)?)),
            },
            0xD if n==0 =>Ok(Self::Draw { registry, registry2 }),
            _ => Ok(Self::Chip8(Chip8Opcode::decode(opcode)?)),
        }

        // match Chip8Opcode::decode(opcode){
        //     Ok(opcode)=>Ok(Self::Chip8(opcode)),
        //     Err(_)=>{
        //         Err(crate::UnkownOpCodeErr(0))

        //     }
        // }
    }
    fn execute_opcode<O: Opcode + 'static>(self, emu: &mut crate::ChipEmulator<O>) -> bool {
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
            Chip8Opcode::JumpPlusV0 { address }=>{
                emu.set_program_counter(address as usize + emu.get_v(((address & 0xF00) >> 8) as u8) as usize);
                increase_program_counter = false
            }

                _ => increase_program_counter = opcode.execute_opcode(emu),
            },

            Self::LowerScreenResolution=>emu.resize_display(2048),
            Self::RaiseScreenResolution=>emu.resize_display(8192),
            Self::Draw { .. }=>unimplemented!(),
            Self::ScrollRight=>{
                let (width,_) = emu.get_display_size();
                // let width  = 64;
                for row in emu.get_display_mut().chunks_mut(width) {
                    // let len = row.len();
                    
                    row.copy_within(0..width-4, 4);
                    row[..4].fill(0);
                    
                }
            },
            Self::ScrollDown=>()
        }
        increase_program_counter
    }
    fn useless_opcode() -> Self {
        Self::Chip8(Chip8Opcode::useless_opcode())
    }
}

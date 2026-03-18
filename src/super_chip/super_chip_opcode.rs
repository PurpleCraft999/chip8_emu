use crate::{Chip8OpCode, emulator::Opcode};

pub enum SuperChipOpcode{
    Chip8(Chip8OpCode),
    A
}
impl Opcode for SuperChipOpcode{
    fn decode(opcode: u16) -> Result<Self, crate::UnkownOpCodeErr> {
        match Chip8OpCode::decode(opcode){
            Ok(opcode)=>Ok(Self::Chip8(opcode)),
            Err(_)=>{
                Err(crate::UnkownOpCodeErr(0))





            }
        }
    }
    fn execute_opcode<O: Opcode + 'static>(self, _emu: &mut crate::ChipEmulator<O>) -> bool {
        false
    }
    fn useless_opcode() -> Self {
        Self::A
    }
}
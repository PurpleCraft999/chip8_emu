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
    ///8xy2
    BitAnd {
        registry: u8,
        registry2: u8,
    },
    ///8xy3
    BitXor {
        registry: u8,
        registry2: u8,
    },
    ///8xy4
    AddReg {
        registry: u8,
        registry2: u8,
    },
    ///8xy5
    SubReg {
        registry: u8,
        registry2: u8,
    },
    ///8xy6
    ShiftRight {
        registry: u8,
        registry2: u8,
    },
    ///8xy7
    SubRegYX {
        registry: u8,
        registry2: u8,
    },
    ///8xyE
    ShiftLeft {
        registry: u8,
        registry2: u8,
    },
    //below here needs tests
    /// Fx65
    ///loads from memory starting from i into registries including max_registry
    LoadMemIntoRegs {
        max_registry: u8,
    },
    ///Fx55
    ///stores the registries 0..=max_registry in memory starting from i
    StoreRegsIntoMem {
        max_registry: u8,
    },
    ///Fx33
    ///store binary-coded decimal representation of vX to memory at i, i + 1 and i + 2
    StoreVXAsBinary {
        registry: u8,
    },
    ///Fx1E
    AddAssignI {
        registry: u8,
    },
    ///ExA1
    SkipIfNotPressed {
        registry: u8,
    },
    ///Ex9E
    SkipIfPressed {
        registry: u8,
    },
    ///Fx07
    PutDelayInRegX {
        registry: u8,
    },
    ///Fx15
    ///sets the delay timer to the value of the register
    SetDelayTimer {
        registry: u8,
    },
    ///Fx18
    ///sets the sound timer to the value of the register
    SetSoundTimer {
        registry: u8,
    },
    ///Fx0A
    ///waits for a key press then stores it in the registey
    WaitForKey {
        registry: u8,
    },
    ///0xB
    JumpPlusV0 {
        address: u16,
    },
    ///0xC
    Random {
        registry: u8,
        value: u8,
    },
    ///Fx29
    LoadFont {
        registry: u8,
    },
    ///0x0nnn
    ///dont even have a way to get it
    JumpToSystemAddress{address:u16}
}
impl Chip8OpCode {
    pub fn decode(opcode: u16) -> Result<Self, UnkownOpCodeErr> {
        //gets the any memory address if it has one
        // nnn
        let address = opcode & 0x0FFF;
        //gets the registry number
        let registry = ((opcode & 0x0F00) >> 8) as u8;
        //gets the value
        // kk
        let value = (opcode & 0x00FF) as u8;
        //y
        let registry2 = ((opcode & 0x00F0) >> 4) as u8;
        let n = (opcode & 0x000F) as u8;

        //get just the operator type
        match (opcode & 0xF000) >> 12 {
            //op type is 0 it must be one of these
            0x0 => match opcode {
                0xE0 => Ok(Self::ClearScreen),
                0xEE => Ok(Self::Return),
                //this is a blank address
                0  =>Err(UnkownOpCodeErr(opcode)),
                _ => Ok(Self::JumpToSystemAddress { address }),
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
            0xF if registry2 == 2 && n == 9 => Ok(Self::LoadFont { registry }),
            0xB => Ok(Self::JumpPlusV0 { address }),
            0xC => Ok(Self::Random { registry, value }),
            _ => Err(UnkownOpCodeErr(opcode)),
        }
    }
}

pub struct UnkownOpCodeErr(pub(crate) u16);

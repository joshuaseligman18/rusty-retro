use num_enum::TryFromPrimitive;

pub struct Instruction {
    pub opcode: u8,
    pub decoded: DecodedOpcode,
}

impl From<u8> for Instruction {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            opcode: value,
            decoded: value.into(),
        }
    }
}

#[derive(Debug, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum R8 {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    HLMem = 0b110,
    A = 0b111,
}

#[derive(Debug, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum R16 {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

#[derive(Debug, TryFromPrimitive, Clone)]
#[repr(u8)]
pub enum R16Mem {
    BC = 0b00,
    DE = 0b01,
    HLInc = 0b10,
    HLDec = 0b11,
}

pub struct DecodedOpcode {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl DecodedOpcode {
    #[inline]
    pub fn p(&self) -> u8 {
        (self.y >> 1) & 0b11
    }

    #[inline]
    pub fn q(&self) -> u8 {
        self.y & 0b1
    }

    #[inline]
    pub fn r8_y(&self) -> R8 {
        R8::try_from(self.y).unwrap()
    }

    #[inline]
    pub fn r8_z(&self) -> R8 {
        R8::try_from(self.z).unwrap()
    }

    #[inline]
    pub fn r16_p(&self) -> R16 {
        R16::try_from(self.p()).unwrap()
    }

    #[inline]
    pub fn r16mem_p(&self) -> R16Mem {
        R16Mem::try_from(self.p()).unwrap()
    }
}

impl From<u8> for DecodedOpcode {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            x: (value >> 6) & 0b11,
            y: (value >> 3) & 0b111,
            z: value & 0b111,
        }
    }
}

use num_enum::TryFromPrimitive;

pub struct Instruction {
    pub opcode: u8,
    pub decoded: DecodedOpcode,
}

impl Instruction {
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            decoded: DecodedOpcode::from(opcode),
        }
    }
}

#[derive(Debug, TryFromPrimitive)]
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

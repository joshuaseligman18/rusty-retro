use crate::gb::cpu::instruction::R8;

#[derive(Debug)]
pub enum Register8Bit {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl TryFrom<R8> for Register8Bit {
    type Error = &'static str;

    fn try_from(value: R8) -> Result<Self, Self::Error> {
        match value {
            R8::B => Ok(Self::B),
            R8::C => Ok(Self::C),
            R8::D => Ok(Self::D),
            R8::E => Ok(Self::E),
            R8::H => Ok(Self::H),
            R8::L => Ok(Self::L),
            R8::HLMem => Err("R8::HLMem cannot be converted into Register8Bit"),
            R8::A => Ok(Self::A),
        }
    }
}

#[derive(Debug)]
pub enum Register16Bit {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

pub struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            sp: 0x0000,
            pc: 0x0000,
        }
    }

    pub fn get_register_8bit(&self, register: Register8Bit) -> u8 {
        match register {
            Register8Bit::A => self.a,
            Register8Bit::B => self.b,
            Register8Bit::C => self.c,
            Register8Bit::D => self.d,
            Register8Bit::E => self.e,
            Register8Bit::H => self.h,
            Register8Bit::L => self.l,
        }
    }

    pub fn set_register_8bit(&mut self, register: Register8Bit, val: u8) {
        match register {
            Register8Bit::A => self.a = val,
            Register8Bit::B => self.b = val,
            Register8Bit::C => self.c = val,
            Register8Bit::D => self.d = val,
            Register8Bit::E => self.e = val,
            Register8Bit::H => self.h = val,
            Register8Bit::L => self.l = val,
        }
    }

    pub fn get_register_16bit(&self, register: Register16Bit) -> u16 {
        match register {
            Register16Bit::AF => unimplemented!("Get register AF"),
            Register16Bit::BC => {
                (self.get_register_8bit(Register8Bit::B) as u16) << 8
                    | (self.get_register_8bit(Register8Bit::C) as u16)
            }
            Register16Bit::DE => {
                (self.get_register_8bit(Register8Bit::D) as u16) << 8
                    | (self.get_register_8bit(Register8Bit::E) as u16)
            }
            Register16Bit::HL => {
                (self.get_register_8bit(Register8Bit::H) as u16) << 8
                    | (self.get_register_8bit(Register8Bit::L) as u16)
            }
            Register16Bit::SP => self.sp,
            Register16Bit::PC => self.pc,
        }
    }

    pub fn set_register_16bit(&mut self, register: Register16Bit, val: u16) {
        let low = (val & 0x00FF) as u8;
        let high = (val >> 8) as u8;

        match register {
            Register16Bit::AF => unimplemented!("Get register AF"),
            Register16Bit::BC => {
                self.set_register_8bit(Register8Bit::B, high);
                self.set_register_8bit(Register8Bit::C, low);
            }
            Register16Bit::DE => {
                self.set_register_8bit(Register8Bit::D, high);
                self.set_register_8bit(Register8Bit::E, low);
            }
            Register16Bit::HL => {
                self.set_register_8bit(Register8Bit::H, high);
                self.set_register_8bit(Register8Bit::L, low);
            }
            Register16Bit::SP => self.sp = val,
            Register16Bit::PC => self.pc = val,
        }
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_register_8bit() {
        let mut registers = Registers::new();
        registers.set_register_8bit(Register8Bit::B, 0x42);
        assert_eq!(registers.get_register_8bit(Register8Bit::B), 0x42);
    }

    #[test]
    fn test_set_register_16bit() {
        let mut registers = Registers::new();
        registers.set_register_16bit(Register16Bit::BC, 0x1218);
        assert_eq!(registers.get_register_16bit(Register16Bit::BC), 0x1218);
        assert_eq!(registers.get_register_8bit(Register8Bit::B), 0x12);
        assert_eq!(registers.get_register_8bit(Register8Bit::C), 0x18);
    }
}

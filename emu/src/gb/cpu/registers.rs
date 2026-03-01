use bitflags::bitflags;

use crate::gb::cpu::{
    alu::AluResultInfo,
    instruction::{R8, R16, R16Mem, R16Stk},
};

#[derive(Debug, Clone)]
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

    #[inline]
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

#[derive(Debug, Clone)]
pub enum Register16Bit {
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

impl From<R16> for Register16Bit {
    #[inline]
    fn from(value: R16) -> Self {
        match value {
            R16::BC => Self::BC,
            R16::DE => Self::DE,
            R16::HL => Self::HL,
            R16::SP => Self::SP,
        }
    }
}

impl From<R16Mem> for Register16Bit {
    #[inline]
    fn from(value: R16Mem) -> Self {
        match value {
            R16Mem::BC => Self::BC,
            R16Mem::DE => Self::DE,
            R16Mem::HLInc | R16Mem::HLDec => Self::HL,
        }
    }
}

impl From<R16Stk> for Register16Bit {
    #[inline]
    fn from(value: R16Stk) -> Self {
        match value {
            R16Stk::BC => Self::BC,
            R16Stk::DE => Self::DE,
            R16Stk::HL => Self::HL,
            R16Stk::AF => Self::AF,
        }
    }
}

bitflags! {
    #[derive(Clone)]
    pub struct FlagsRegister: u8 {
        const Zero = 0b10000000;
        const Subtraction = 0b01000000;
        const HalfCarry = 0b00100000;
        const Carry = 0b00010000;
    }
}

pub struct Registers {
    a: u8,
    f: FlagsRegister,
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
            f: FlagsRegister::empty(),
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

    #[inline]
    pub fn get_flags(&self) -> FlagsRegister {
        self.f.clone()
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    pub fn get_register_16bit(&self, register: Register16Bit) -> u16 {
        match register {
            Register16Bit::AF => (self.a as u16) << 8 | ((self.f.bits() & 0xF0) as u16),
            Register16Bit::BC => (self.b as u16) << 8 | (self.c as u16),
            Register16Bit::DE => (self.d as u16) << 8 | (self.e as u16),
            Register16Bit::HL => (self.h as u16) << 8 | (self.l as u16),
            Register16Bit::SP => self.sp,
            Register16Bit::PC => self.pc,
        }
    }

    #[inline]
    pub fn set_register_16bit(&mut self, register: Register16Bit, val: u16) {
        let low = (val & 0x00FF) as u8;
        let high = (val >> 8) as u8;

        match register {
            Register16Bit::AF => {
                self.a = high;
                self.f = FlagsRegister::from_bits_truncate(low & 0xF0);
            }
            Register16Bit::BC => {
                self.b = high;
                self.c = low;
            }
            Register16Bit::DE => {
                self.d = high;
                self.e = low;
            }
            Register16Bit::HL => {
                self.h = high;
                self.l = low;
            }
            Register16Bit::SP => self.sp = val,
            Register16Bit::PC => self.pc = val,
        }
    }

    pub fn set_flags_from_alu_res_info(&mut self, res_info: &AluResultInfo, mask: FlagsRegister) {
        self.f.remove(mask.clone());
        self.f
            .insert(FlagsRegister::from_bits_truncate(res_info.bits()) & mask);
    }

    pub fn set_flags(&mut self, new_flags: &FlagsRegister, mask: FlagsRegister) {
        self.f.remove(mask.clone());
        self.f
            .insert(FlagsRegister::from_bits_truncate(new_flags.bits()) & mask);
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

    #[test]
    fn test_set_flags_from_alu_res_info() {
        let mut registers = Registers::new();
        registers.f.set(FlagsRegister::Zero, true);
        registers.f.set(FlagsRegister::Subtraction, true);

        let mut res_info = AluResultInfo::empty();
        res_info.set(AluResultInfo::Carry, true);
        res_info.set(AluResultInfo::HalfCarry, false);
        res_info.set(AluResultInfo::Zero, false);
        res_info.set(AluResultInfo::Subtraction, false);
        registers.set_flags_from_alu_res_info(
            &res_info,
            FlagsRegister::Subtraction | FlagsRegister::Carry | FlagsRegister::HalfCarry,
        );

        assert!(registers.f.contains(FlagsRegister::Zero));
        assert!(registers.f.contains(FlagsRegister::Carry));
        assert!(!registers.f.contains(FlagsRegister::HalfCarry));
        assert!(!registers.f.contains(FlagsRegister::Subtraction));
    }

    #[test]
    fn test_set_flags() {
        let mut registers = Registers::new();
        registers.f.set(FlagsRegister::Zero, true);
        registers.f.set(FlagsRegister::Subtraction, true);

        let mut new_flags = FlagsRegister::empty();
        new_flags.set(FlagsRegister::Carry, true);
        new_flags.set(FlagsRegister::HalfCarry, false);
        new_flags.set(FlagsRegister::Zero, false);
        new_flags.set(FlagsRegister::Subtraction, false);
        registers.set_flags(
            &new_flags,
            FlagsRegister::Subtraction | FlagsRegister::Carry | FlagsRegister::HalfCarry,
        );

        assert!(registers.f.contains(FlagsRegister::Zero));
        assert!(registers.f.contains(FlagsRegister::Carry));
        assert!(!registers.f.contains(FlagsRegister::HalfCarry));
        assert!(!registers.f.contains(FlagsRegister::Subtraction));
    }
}

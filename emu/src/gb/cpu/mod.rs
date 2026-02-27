mod registers;

use std::{cell::RefCell, rc::Rc};

use crate::{gb::cpu::registers::{Registers, Register16Bit}, ram::Ram};

pub struct LR35902 {
    ram: Rc<RefCell<Ram<u8>>>,
    registers: registers::Registers,
}

impl LR35902 {
    pub fn new(sys_ram: Rc<RefCell<Ram<u8>>>) -> Self {
        Self {
            ram: sys_ram,
            registers: Registers::new()
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_imm8();
    }

    fn fetch_imm8(&mut self) -> u8 {
        let data = self.ram.borrow().read(self.registers.get_register_16bit(Register16Bit::PC) as usize);
        self.registers.set_register_16bit(Register16Bit::PC, self.registers.get_register_16bit(Register16Bit::PC) + 1);
        data
    }

    fn fetch_imm16(&mut self) -> u16 {
        let low = self.fetch_imm8();
        let high = self.fetch_imm8();
        (high as u16) << 8 | (low as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_test_cpu() -> LR35902 {
        let test_ram = Rc::new(RefCell::new(Ram::new(0x10000))); 
        LR35902::new(Rc::clone(&test_ram))
    }

    #[test]
    fn test_fetch_imm8() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x0000, 0x42);

        let data = test_cpu.fetch_imm8();
        assert_eq!(data, 0x42);
        assert_eq!(test_cpu.registers.get_register_16bit(Register16Bit::PC), 0x0001);
    }

    #[test]
    fn test_fetch_imm16() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x0000, 0x18);
        test_cpu.ram.borrow_mut().write(0x0001, 0x12);

        let data = test_cpu.fetch_imm16();
        assert_eq!(data, 0x1218);
        assert_eq!(test_cpu.registers.get_register_16bit(Register16Bit::PC), 0x0002);
    }
}

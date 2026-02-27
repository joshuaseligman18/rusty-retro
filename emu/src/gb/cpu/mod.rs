mod instruction;
mod registers;

use std::{cell::RefCell, rc::Rc};

use crate::{
    gb::cpu::{
        instruction::Instruction,
        registers::{Register8Bit, Register16Bit, Registers},
    },
    ram::Ram,
};

pub struct LR35902 {
    ram: Rc<RefCell<Ram<u8>>>,
    registers: registers::Registers,
}

impl LR35902 {
    pub fn new(sys_ram: Rc<RefCell<Ram<u8>>>) -> Self {
        Self {
            ram: sys_ram,
            registers: Registers::new(),
        }
    }

    pub fn step(&mut self) {
        let opcode = self.fetch_imm8();
        let instruction = Instruction::from(opcode);
        match instruction.decoded.x {
            0b00 => self.handle_block0(&instruction),
            0b01 => self.handle_block1(&instruction),
            0b10 => self.handle_block2(&instruction),
            0b11 => self.handle_block3(&instruction),
            _ => unreachable!("Invalid decoded x value"),
        }
    }

    fn fetch_imm8(&mut self) -> u8 {
        let data = self
            .ram
            .borrow()
            .read(self.registers.get_register_16bit(Register16Bit::PC) as usize);
        self.registers.set_register_16bit(
            Register16Bit::PC,
            self.registers.get_register_16bit(Register16Bit::PC) + 1,
        );
        data
    }

    fn fetch_imm16(&mut self) -> u16 {
        let low = self.fetch_imm8();
        let high = self.fetch_imm8();
        (high as u16) << 8 | (low as u16)
    }

    fn handle_block0(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b00);
    }

    fn handle_block1(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b01);

        if instruction.decoded.y == 0b110 && instruction.decoded.z == 0b110 {
            unimplemented!("HALT");
        }

        let src: u8 = match Register8Bit::try_from(instruction.decoded.r8_z()) {
            Ok(reg) => self.registers.get_register_8bit(reg),
            Err(_) => self
                .ram
                .borrow()
                .read(self.registers.get_register_16bit(Register16Bit::HL) as usize),
        };

        match Register8Bit::try_from(instruction.decoded.r8_y()) {
            Ok(reg) => self.registers.set_register_8bit(reg, src),
            Err(_) => self.ram.borrow_mut().write(
                self.registers.get_register_16bit(Register16Bit::HL) as usize,
                src,
            ),
        }
    }

    fn handle_block2(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b10);
    }

    fn handle_block3(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b11);
    }
}

#[cfg(test)]
mod tests {
    use crate::gb::cpu::registers::Register16Bit;

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
        assert_eq!(
            test_cpu.registers.get_register_16bit(Register16Bit::PC),
            0x0001
        );
    }

    #[test]
    fn test_fetch_imm16() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x0000, 0x18);
        test_cpu.ram.borrow_mut().write(0x0001, 0x12);

        let data = test_cpu.fetch_imm16();
        assert_eq!(data, 0x1218);
        assert_eq!(
            test_cpu.registers.get_register_16bit(Register16Bit::PC),
            0x0002
        );
    }

    #[test]
    fn test_handle_block1_reg_reg() {
        let mut test_cpu = init_test_cpu();

        let opcode = 0b01000001;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x42);
        test_cpu.registers.set_register_8bit(Register8Bit::C, 0x18);

        test_cpu.handle_block1(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::B), 0x18);
    }

    #[test]
    fn test_handle_block1_reg_mem() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x2112, 0x18);

        let opcode = 0b01000110;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x42);
        test_cpu
            .registers
            .set_register_16bit(Register16Bit::HL, 0x2112);

        test_cpu.handle_block1(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::B), 0x18);
    }

    #[test]
    fn test_handle_block1_mem_reg() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x2112, 0x42);

        let opcode = 0b01110000;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x18);
        test_cpu
            .registers
            .set_register_16bit(Register16Bit::HL, 0x2112);

        test_cpu.handle_block1(&instruction);
        assert_eq!(test_cpu.ram.borrow().read(0x2112), 0x18);
    }

    #[test]
    #[should_panic]
    fn test_handle_block1_halt() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x2112, 0x42);

        let opcode = 0b0111000;
        let instruction = Instruction::from(opcode);
        test_cpu.handle_block1(&instruction);
    }
}

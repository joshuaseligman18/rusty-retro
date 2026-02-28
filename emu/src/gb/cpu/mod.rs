mod alu;
mod instruction;
mod registers;

use std::{cell::RefCell, rc::Rc};

use crate::{
    gb::cpu::{
        alu::{
            AluResultInfo, add_with_carry, bitwise_and, bitwise_or, bitwise_xor, rotate_left,
            rotate_left_through_carry, rotate_right, rotate_right_through_carry,
            subtract_with_carry,
        },
        instruction::{Instruction, R16Mem},
        registers::{FlagsRegister, Register8Bit, Register16Bit, Registers},
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
            self.registers
                .get_register_16bit(Register16Bit::PC)
                .wrapping_add(1),
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

        match (
            instruction.decoded.p(),
            instruction.decoded.q(),
            instruction.decoded.z,
        ) {
            // nop
            (_, 0b0, 0b000) => {}
            // ld r16, imm16
            (_, 0b0, 0b001) => {
                let imm = self.fetch_imm16();
                let dest: Register16Bit = instruction.decoded.r16_p().into();
                self.registers.set_register_16bit(dest, imm);
            }
            // ld [r16mem], a
            (_, 0b0, 0b010) => {
                let dest_reg: R16Mem = instruction.decoded.r16mem_p();
                let dest_addr = self.registers.get_register_16bit(dest_reg.clone().into());
                let a = self.registers.get_register_8bit(Register8Bit::A);
                self.ram.borrow_mut().write(dest_addr as usize, a);

                match dest_reg {
                    R16Mem::HLInc => self
                        .registers
                        .set_register_16bit(Register16Bit::HL, dest_addr.wrapping_add(1)),
                    R16Mem::HLDec => self
                        .registers
                        .set_register_16bit(Register16Bit::HL, dest_addr.wrapping_sub(1)),
                    _ => {}
                }
            }
            // ld a, [r16mem]
            (_, 0b1, 0b010) => {
                let src_reg: R16Mem = instruction.decoded.r16mem_p();
                let src_addr = self.registers.get_register_16bit(src_reg.clone().into());
                let src_data = self.ram.borrow().read(src_addr as usize);
                self.registers.set_register_8bit(Register8Bit::A, src_data);

                match src_reg {
                    R16Mem::HLInc => self
                        .registers
                        .set_register_16bit(Register16Bit::HL, src_addr.wrapping_add(1)),
                    R16Mem::HLDec => self
                        .registers
                        .set_register_16bit(Register16Bit::HL, src_addr.wrapping_sub(1)),
                    _ => {}
                }
            }
            // ld [imm16], sp
            (0b00, 0b1, 0b000) => {
                let dest_addr = self.fetch_imm16();
                let sp = self.registers.get_register_16bit(Register16Bit::SP);
                self.ram
                    .borrow_mut()
                    .write(dest_addr as usize, (sp & 0xFF) as u8);
                self.ram
                    .borrow_mut()
                    .write(dest_addr.wrapping_add(1) as usize, (sp >> 8) as u8);
            }
            // inc r16
            (_, 0b0, 0b011) => {
                let cur_reg_val = self
                    .registers
                    .get_register_16bit(instruction.decoded.r16_p().into());
                self.registers.set_register_16bit(
                    instruction.decoded.r16_p().into(),
                    cur_reg_val.wrapping_add(1),
                );
            }
            // dec r16
            (_, 0b1, 0b011) => {
                let cur_reg_val = self
                    .registers
                    .get_register_16bit(instruction.decoded.r16_p().into());
                self.registers.set_register_16bit(
                    instruction.decoded.r16_p().into(),
                    cur_reg_val.wrapping_sub(1),
                );
            }
            // add hl, r16
            (_, 0b1, 0b001) => {
                let hl = self.registers.get_register_16bit(Register16Bit::HL);
                let add_reg_val = self
                    .registers
                    .get_register_16bit(instruction.decoded.r16_p().into());
                let lower = add_with_carry((hl & 0xFF) as u8, (add_reg_val & 0xFF) as u8, false);
                let upper = add_with_carry(
                    (hl >> 8) as u8,
                    (add_reg_val >> 8) as u8,
                    lower.info.contains(AluResultInfo::Carry),
                );
                let new_hl = ((upper.res as u16) << 8) | (lower.res as u16);
                self.registers.set_register_16bit(Register16Bit::HL, new_hl);
                self.registers.set_flags_from_alu_res_info(
                    &upper.info,
                    FlagsRegister::Carry | FlagsRegister::HalfCarry | FlagsRegister::Subtraction,
                );
            }
            // inc r8
            (_, _, 0b100) => {
                let reg_or_mem = Register8Bit::try_from(instruction.decoded.r8_y());
                let cur_val: u8 = match reg_or_mem.clone() {
                    Ok(reg) => self.registers.get_register_8bit(reg),
                    Err(_) => self
                        .ram
                        .borrow()
                        .read(self.registers.get_register_16bit(Register16Bit::HL) as usize),
                };

                let inc_val = add_with_carry(cur_val, 1, false);

                match reg_or_mem {
                    Ok(reg) => self.registers.set_register_8bit(reg, inc_val.res),
                    Err(_) => self.ram.borrow_mut().write(
                        self.registers.get_register_16bit(Register16Bit::HL) as usize,
                        inc_val.res,
                    ),
                };
                self.registers.set_flags_from_alu_res_info(
                    &inc_val.info,
                    FlagsRegister::Zero | FlagsRegister::Subtraction | FlagsRegister::HalfCarry,
                );
            }
            // dec r8
            (_, _, 0b101) => {
                let reg_or_mem = Register8Bit::try_from(instruction.decoded.r8_y());
                let cur_val: u8 = match reg_or_mem.clone() {
                    Ok(reg) => self.registers.get_register_8bit(reg),
                    Err(_) => self
                        .ram
                        .borrow()
                        .read(self.registers.get_register_16bit(Register16Bit::HL) as usize),
                };

                let dec_val = subtract_with_carry(cur_val, 1, false);

                match reg_or_mem {
                    Ok(reg) => self.registers.set_register_8bit(reg, dec_val.res),
                    Err(_) => self.ram.borrow_mut().write(
                        self.registers.get_register_16bit(Register16Bit::HL) as usize,
                        dec_val.res,
                    ),
                };
                self.registers.set_flags_from_alu_res_info(
                    &dec_val.info,
                    FlagsRegister::Zero | FlagsRegister::Subtraction | FlagsRegister::HalfCarry,
                );
            }
            // ld r8, imm8
            (_, _, 0b110) => {
                let src = self.fetch_imm8();
                match Register8Bit::try_from(instruction.decoded.r8_y()) {
                    Ok(reg) => self.registers.set_register_8bit(reg, src),
                    Err(_) => self.ram.borrow_mut().write(
                        self.registers.get_register_16bit(Register16Bit::HL) as usize,
                        src,
                    ),
                };
            }
            // rlca
            (0b00, 0b0, 0b111) => {
                let res = rotate_left(self.registers.get_register_8bit(Register8Bit::A));
                self.registers.set_register_8bit(Register8Bit::A, res.res);
                self.registers
                    .set_flags_from_alu_res_info(&res.info, FlagsRegister::all());
            }
            // rrca
            (0b00, 0b1, 0b111) => {
                let res = rotate_right(self.registers.get_register_8bit(Register8Bit::A));
                self.registers.set_register_8bit(Register8Bit::A, res.res);
                self.registers
                    .set_flags_from_alu_res_info(&res.info, FlagsRegister::all());
            }
            // rla
            (0b01, 0b0, 0b111) => {
                let res = rotate_left_through_carry(
                    self.registers.get_register_8bit(Register8Bit::A),
                    self.registers.get_flags().contains(FlagsRegister::Carry),
                );
                self.registers.set_register_8bit(Register8Bit::A, res.res);
                self.registers
                    .set_flags_from_alu_res_info(&res.info, FlagsRegister::all());
            }
            // rra
            (0b01, 0b1, 0b111) => {
                let res = rotate_right_through_carry(
                    self.registers.get_register_8bit(Register8Bit::A),
                    self.registers.get_flags().contains(FlagsRegister::Carry),
                );
                self.registers.set_register_8bit(Register8Bit::A, res.res);
                self.registers
                    .set_flags_from_alu_res_info(&res.info, FlagsRegister::all());
            }
            (_, _, _) => unimplemented!(),
        }
    }

    fn handle_block1(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b01);

        // halt
        if instruction.decoded.y == 0b110 && instruction.decoded.z == 0b110 {
            unimplemented!("HALT");
        }

        // ld r8, r8
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

        let a: u8 = self.registers.get_register_8bit(Register8Bit::A);
        let f: FlagsRegister = self.registers.get_flags();

        let src: u8 = match Register8Bit::try_from(instruction.decoded.r8_z()) {
            Ok(reg) => self.registers.get_register_8bit(reg),
            Err(_) => self
                .ram
                .borrow()
                .read(self.registers.get_register_16bit(Register16Bit::HL) as usize),
        };

        let alu_res = match instruction.decoded.y {
            // add a, r8
            0b000 => add_with_carry(a, src, false),
            // adc a, r8
            0b001 => add_with_carry(a, src, f.contains(FlagsRegister::Carry)),
            // sub a, r8
            0b010 => subtract_with_carry(a, src, false),
            // sbc a, r8
            0b011 => subtract_with_carry(a, src, f.contains(FlagsRegister::Carry)),
            // and a, r8
            0b100 => bitwise_and(a, src),
            // xor a, r8
            0b101 => bitwise_xor(a, src),
            // or a, r8
            0b110 => bitwise_or(a, src),
            // cp a, r8
            0b111 => subtract_with_carry(a, src, false),
            _ => unreachable!(),
        };

        if instruction.decoded.y != 0b111 {
            self.registers
                .set_register_8bit(Register8Bit::A, alu_res.res);
        }
        self.registers
            .set_flags_from_alu_res_info(&alu_res.info, FlagsRegister::all());
    }

    fn handle_block3(&mut self, instruction: &Instruction) {
        assert_eq!(instruction.decoded.x, 0b11);
    }
}

#[cfg(test)]
mod tests {
    use bitflags::Flags;

    use crate::gb::cpu::{alu::AluResultInfo, registers::Register16Bit};

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

    #[test]
    fn test_handle_block2_reg() {
        let mut test_cpu = init_test_cpu();

        let opcode = 0b10000000;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::A, 0x18);
        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x0D);

        test_cpu.handle_block2(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::A), 0x25);
        assert!(!test_cpu.registers.get_flags().contains(FlagsRegister::Zero));
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Carry)
        );
        assert!(
            test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::HalfCarry)
        );
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Subtraction)
        );
    }

    #[test]
    fn test_handle_block2_carry_flag() {
        let mut test_cpu = init_test_cpu();

        let opcode = 0b10001000;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::A, 0x18);
        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x0D);
        test_cpu
            .registers
            .set_flags_from_alu_res_info(&AluResultInfo::Carry, FlagsRegister::all());

        test_cpu.handle_block2(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::A), 0x26);
        assert!(!test_cpu.registers.get_flags().contains(FlagsRegister::Zero));
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Carry)
        );
        assert!(
            test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::HalfCarry)
        );
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Subtraction)
        );
    }

    #[test]
    fn test_handle_block2_mem() {
        let mut test_cpu = init_test_cpu();
        test_cpu.ram.borrow_mut().write(0x2112, 0x0D);

        let opcode = 0b10000110;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::A, 0x18);
        test_cpu
            .registers
            .set_register_16bit(Register16Bit::HL, 0x2112);

        test_cpu.handle_block2(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::A), 0x25);
        assert!(!test_cpu.registers.get_flags().contains(FlagsRegister::Zero));
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Carry)
        );
        assert!(
            test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::HalfCarry)
        );
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Subtraction)
        );
    }

    #[test]
    fn test_handle_block2_cp() {
        let mut test_cpu = init_test_cpu();

        let opcode = 0b10111000;
        let instruction = Instruction::from(opcode);

        test_cpu.registers.set_register_8bit(Register8Bit::A, 0x18);
        test_cpu.registers.set_register_8bit(Register8Bit::B, 0x0D);

        test_cpu.handle_block2(&instruction);
        assert_eq!(test_cpu.registers.get_register_8bit(Register8Bit::A), 0x18);
        assert!(!test_cpu.registers.get_flags().contains(FlagsRegister::Zero));
        assert!(
            !test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Carry)
        );
        assert!(
            test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::HalfCarry)
        );
        assert!(
            test_cpu
                .registers
                .get_flags()
                .contains(FlagsRegister::Subtraction)
        );
    }
}

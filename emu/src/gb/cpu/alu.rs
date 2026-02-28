use bitflags::bitflags;

bitflags! {
    pub struct BitOutput: u8 {
        const Sum = 0b01;
        const Carry = 0b10;
    }

    pub struct AluResultInfo: u8 {
        const Zero = 0b10000000;
        const Subtraction = 0b01000000;
        const HalfCarry = 0b00100000;
        const Carry = 0b00010000;
    }
}

pub struct AluResult {
    pub res: u8,
    pub info: AluResultInfo,
}

pub fn add_with_carry(num1: u8, num2: u8, carry: bool) -> AluResult {
    let (intermediate, carry1) = num1.overflowing_add(num2);
    let (result, carry2) = intermediate.overflowing_add(carry as u8);

    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Zero, result == 0);
    info.set(AluResultInfo::Subtraction, false);

    let half_carry = (num1 & 0xF) + (num2 & 0xF) + (carry as u8) > 0xF;
    info.set(AluResultInfo::HalfCarry, half_carry);
    info.set(AluResultInfo::Carry, carry1 || carry2);

    AluResult { res: result, info }
}

pub fn subtract_with_carry(num1: u8, num2: u8, carry: bool) -> AluResult {
    let (intermediate, borrow1) = num1.overflowing_sub(num2);
    let (result, borrow2) = intermediate.overflowing_sub(carry as u8);

    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Zero, result == 0);
    info.set(AluResultInfo::Subtraction, true);

    let half_borrow = (num1 & 0xF) < (num2 & 0xF) + (carry as u8);
    info.set(AluResultInfo::HalfCarry, half_borrow);
    info.set(AluResultInfo::Carry, borrow1 || borrow2);

    AluResult { res: result, info }
}

pub fn bitwise_and(num1: u8, num2: u8) -> AluResult {
    let res = num1 & num2;
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Zero, res == 0);
    info.set(AluResultInfo::Subtraction, false);
    info.set(AluResultInfo::HalfCarry, true);
    info.set(AluResultInfo::Carry, false);

    AluResult { res, info }
}

pub fn bitwise_xor(num1: u8, num2: u8) -> AluResult {
    let res = num1 ^ num2;
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Zero, res == 0);
    info.set(AluResultInfo::Subtraction, false);
    info.set(AluResultInfo::HalfCarry, false);
    info.set(AluResultInfo::Carry, false);

    AluResult { res, info }
}

pub fn bitwise_or(num1: u8, num2: u8) -> AluResult {
    let res = num1 | num2;
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Zero, res == 0);
    info.set(AluResultInfo::Subtraction, false);
    info.set(AluResultInfo::HalfCarry, false);
    info.set(AluResultInfo::Carry, false);

    AluResult { res, info }
}

pub fn rotate_left(num: u8) -> AluResult {
    let res = num.rotate_left(1);
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Carry, (num >> 7) & 0b1 == 1);
    AluResult { res, info }
}

pub fn rotate_right(num: u8) -> AluResult {
    let res = num.rotate_right(1);
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Carry, (num & 0b1) == 1);
    AluResult { res, info }
}

pub fn rotate_left_through_carry(num: u8, carry: bool) -> AluResult {
    let res = (num << 1) | (carry as u8);
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Carry, (num >> 7) & 0b1 == 1);
    AluResult { res, info }
}

pub fn rotate_right_through_carry(num: u8, carry: bool) -> AluResult {
    let res = (num >> 1) | ((carry as u8) << 7);
    let mut info = AluResultInfo::empty();
    info.set(AluResultInfo::Carry, (num & 0b1) == 1);
    AluResult { res, info }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        //   0001 0010
        // + 0001 0010
        //   ---------
        //   0010 0100
        // 0x24

        let out: AluResult = add_with_carry(0x12, 0x12, false);
        assert_eq!(out.res, 0x24);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_add_carry() {
        //   1111 0101
        // + 1111 0101
        //   ---------
        // 1 1110 1010
        // 0xEA with carry 1

        let out: AluResult = add_with_carry(0xF5, 0xF5, false);
        assert_eq!(out.res, 0xEA);
        assert!(out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_add_half_carry() {
        //   0010 1101
        // + 0010 1101
        //   ---------
        //   0101 1010
        // 0x5A with half carry

        let out: AluResult = add_with_carry(0x2D, 0x2D, false);
        assert_eq!(out.res, 0x5A);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_add_carry_and_half_carry() {
        //   1111 1101
        // + 1111 1101
        //   ---------
        // 1 1111 1010
        // 0xFA with carry and half carry

        let out: AluResult = add_with_carry(0xFD, 0xFD, false);
        assert_eq!(out.res, 0xFA);
        assert!(out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_add_zero_and_carry_and_half_carry() {
        //   1111 1110
        //   0000 0001
        // + 0000 0001
        //   ---------
        // 1 0000 0000
        // 0x00 with carry and half carry

        let out: AluResult = add_with_carry(0xFE, 0x01, true);
        assert_eq!(out.res, 0x00);
        assert!(out.info.contains(AluResultInfo::Carry));
        assert!(out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_subtract() {
        //   0001 0101
        // - 0000 0101
        //   ---------
        //   0001 0000
        // 0x10

        let out = subtract_with_carry(0x15, 0x05, false);
        assert_eq!(out.res, 0x10);
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::Subtraction));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Carry));
    }

    #[test]
    fn test_subtract_zero() {
        //   0001 0000
        // - 0001 0000
        //   ---------
        //   0000 0000
        // 0x00

        let out = subtract_with_carry(0x10, 0x10, false);
        assert_eq!(out.res, 0x00);
        assert!(out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::Subtraction));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Carry));
    }

    #[test]
    fn test_subtract_borrow_and_half_borrow() {
        //   0000 0101
        // - 0000 1010
        //   ---------
        // 1 1111 1011
        // 0xFB with borrow and half borrow

        let out = subtract_with_carry(0x05, 0x0A, false);
        assert_eq!(out.res, 0xFB);
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::Subtraction));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(out.info.contains(AluResultInfo::Carry));
    }

    #[test]
    fn test_subtract_half_borrow() {
        //   0001 0000
        //   0000 0101
        // - 0000 0001
        //   ---------
        //   0000 1010
        //  0x0A with half borrow

        let out = subtract_with_carry(0x10, 0x05, true);
        assert_eq!(out.res, 0x0A);
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::Subtraction));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Carry));
    }

    #[test]
    fn test_bitwise_and() {
        // 0110 1001
        // 1101 0111
        // ---------
        // 0100 0001

        let out: AluResult = bitwise_and(0x69, 0xD7);
        assert_eq!(out.res, 0x41);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_bitwise_and_zero() {
        // 1010 1010
        // 0101 0101
        // ---------
        // 0000 0000

        let out: AluResult = bitwise_and(0xAA, 0x55);
        assert_eq!(out.res, 0x00);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(out.info.contains(AluResultInfo::Zero));
        assert!(out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_bitwise_xor() {
        // 0100 1001
        // 1101 0111
        // ---------
        // 1001 1110

        let out: AluResult = bitwise_xor(0x49, 0xD7);
        assert_eq!(out.res, 0x9E);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_bitwise_xor_zero() {
        // 1010 1010
        // 1010 1010
        // ---------
        // 0000 0000

        let out: AluResult = bitwise_xor(0xAA, 0xAA);
        assert_eq!(out.res, 0x00);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_bitwise_or() {
        // 0100 1001
        // 1101 0111
        // ---------
        // 1101 1111

        let out: AluResult = bitwise_or(0x49, 0xD7);
        assert_eq!(out.res, 0xDF);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_bitwise_or_zero() {
        // 0000 0000
        // 0000 0000
        // ---------
        // 0000 0000

        let out: AluResult = bitwise_or(0x00, 0x00);
        assert_eq!(out.res, 0x00);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_rotate_left() {
        // 10110010
        // --------
        // 01100101 with carry = 1

        let out: AluResult = rotate_left(0b10110010);
        assert_eq!(out.res, 0b01100101);
        assert!(out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_rotate_right() {
        // 10110010
        // --------
        // 01011001 with carry = 0

        let out: AluResult = rotate_right(0b10110010);
        assert_eq!(out.res, 0b01011001);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_rotate_left_through_carry() {
        // 00110010 C = 1
        // --------
        // 01100101 with carry = 0

        let out: AluResult = rotate_left_through_carry(0b00110010, true);
        assert_eq!(out.res, 0b01100101);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }

    #[test]
    fn test_rotate_right_through_carry() {
        // 10110010 C = 1
        // --------
        // 11011001 with carry = 0

        let out: AluResult = rotate_right_through_carry(0b10110010, true);
        assert_eq!(out.res, 0b11011001);
        assert!(!out.info.contains(AluResultInfo::Carry));
        assert!(!out.info.contains(AluResultInfo::Zero));
        assert!(!out.info.contains(AluResultInfo::HalfCarry));
        assert!(!out.info.contains(AluResultInfo::Subtraction));
    }
}

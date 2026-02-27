use num_traits::{Bounded, PrimInt, sign::Unsigned};

pub trait WordSize: Unsigned + Bounded + PrimInt {}
impl WordSize for u8 {}

pub struct Ram<RamWordSize: WordSize> {
    arr: Vec<RamWordSize>,
}

impl<RamWordSize: WordSize> Ram<RamWordSize> {
    pub fn new(addr_space: usize) -> Self {
        Self {
            arr: vec![RamWordSize::zero(); addr_space],
        }
    }

    pub fn read(&self, addr: usize) -> RamWordSize {
        self.arr[addr]
    }

    pub fn write(&mut self, addr: usize, data: RamWordSize) {
        self.arr[addr] = data;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of_val;

    #[test]
    fn test_create_ram() {
        let my_ram: Ram<u8> = Ram::new(0x10000);
        assert_eq!(my_ram.arr.len(), 0x10000);
        assert_eq!(size_of_val(&my_ram.read(0x0000)), 1);
        assert_eq!(my_ram.read(0x0000), 0);
    }

    #[test]
    fn test_write_ram() {
        let mut my_ram: Ram<u8> = Ram::new(0x10000);
        my_ram.write(0x4242, 0x07);
        assert_eq!(my_ram.read(0x4242), 0x07)
    }
}

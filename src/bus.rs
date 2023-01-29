use num::traits::int::PrimInt;

use crate::memory::Memory;
use crate::rom::Rom;

/**
 * Emulating the actual bus
 * 
 * Read and write operation should take place from here
 */
pub struct Bus<T: PrimInt + std::convert::From<u8>> {
    memory: Memory<T>, // RAM
    other: Vec<T>, // Other storages or devices
    pub secondary_storage: Rom<T>, // ROM 
}

// Constructor like implementation
impl<T: PrimInt + std::convert::From<u8>> Bus<T> {
    
    pub fn new(memory: Memory<T>, other: Vec<T>, secondary_storage: Rom<T>) -> Self {
        Self {
            memory,
            other,
            secondary_storage
        }
    }
}

impl<T: PrimInt + std::convert::From<u8>> Bus<T> {
    pub fn read(&self, address: u16) -> T {
        if address < self.memory.len() as u16 {
            return self.memory[address];
        }
        else if address < self.memory.len() as u16 + self.other.len() as u16 {
            return self.other[(address - self.memory.len() as u16) as usize];
        }
        else if (address as u32) < self.memory.len() as u32 + self.other.len() as u32 + self.secondary_storage.len() as u32 {
            return self.secondary_storage[address - self.memory.len() as u16 - self.other.len() as u16];
        }
        else {
            return T::zero();
        }
    }

    pub fn write(&mut self, address: u16, data: T) -> () {
        if address < self.memory.len() as u16 {
            self.memory[address] = data;
        }
        else if address < self.memory.len() as u16 + self.other.len() as u16 {
            self.other[(address - self.memory.len() as u16) as usize] = data;
        }
        else if (address as u32) < self.memory.len() as u32 + self.other.len() as u32 + self.secondary_storage.len() as u32 {
            self.secondary_storage[address - self.memory.len() as u16 - self.other.len() as u16] = data;
        }
        else {
            panic!("Invalid Write Address");
        }
    }
}

// load ROM implementation
impl<T: PrimInt + std::convert::From<u8>> Bus<T> {

    pub fn load_rom(&mut self, filepath: &str) -> bool {
        self.secondary_storage.load(filepath)
    }
}
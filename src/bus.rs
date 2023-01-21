use crate::memory::Memory;

/**
 * Emulating the actual bus
 * 
 * Read and write operation should take place from here
 */
pub struct Bus<T> {
    memory: Memory<T>, // RAM
    other: Vec<T>, // Other storages or devices
    secondary_storage: Vec<T>, // ROM 
}

// Constructor like implementation
impl Bus<u8> {
    
    pub fn new(memory: Memory<u8>, other: Vec<u8>, secondary_storage: Vec<u8>) -> Self {
        Self {
            memory,
            other,
            secondary_storage
        }
    }
}

impl Bus<u8> {
    pub fn read(&self, address: u16) -> u8 {
        if address > 0 && address < self.memory.len() as u16 {
            return self.memory[address];
        }
        else if address < self.other.len() as u16 {
            return self.other[address as usize];
        }
        else if address < self.secondary_storage.len() as u16 {
            return self.secondary_storage[address as usize];
        }
        else {
            return 0x00;
        }
    }

    pub fn write(&mut self, address: u16, data: u8) -> () {
        if address > 0 && address < self.memory.len() as u16 {
            self.memory[address] = data;
        }
        else if address < self.other.len() as u16 {
            self.other[address as usize] = data;
        }
        else if address < self.secondary_storage.len() as u16 {
            self.secondary_storage[address as usize] = data;
        }
        else {
            panic!("Invalid Write Address");
        }
    }
}


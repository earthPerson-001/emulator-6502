use std::io::{BufReader, Read};
use std::fs::File;

use num::traits::int::PrimInt;

pub struct Rom<T: PrimInt + std::convert::From<u8>> {
    pub rom: Vec<T>,
}

impl<T: PrimInt + std::convert::From<u8>> Rom<T> {
    /**
    Returns the Rom with given size

    Creates a vector filling the given size with zeros

    # Arguments

    * `size_b` - The Rom size in bytes

    */
    pub fn new(size_b: usize) -> Self {
        Self {
            rom: vec![T::zero(); size_b],
        }
    }
}

// overloading [] for read access
impl<T: PrimInt + std::convert::From<u8>> std::ops::Index<u16> for Rom<T> {
    type Output = T;

    fn index(&self, index: u16) -> &T {
        return {
            if index < self.rom.len() as u16 {
                let opt = self.rom.get(index as usize);
                match opt {
                    Some(val) => val,
                    None => panic!("Invalid Read Address"),
                }
            } else {
               panic!("Invalid Read Address");
            }
        };
    }
}

// loading data into Rom
impl<T: PrimInt + std::convert::From<u8>> Rom<T> {

    pub fn load(&mut self, filepath: &str) -> bool {


        let file = File::open(filepath);

        // to place the read file
        let mut buffer_for_rom = vec![0 as u8; self.rom.len()];
        let buffer_size = buffer_for_rom.len();

        match file {
            Ok(opened_file) => {
                    let mut buffered_reader = BufReader::new(opened_file);
                    if let Ok(()) = buffered_reader.read_exact(&mut buffer_for_rom[0..0 + buffer_size]) {

                        // copying the value from buffer to rom
                        for (i, value) in self.rom.iter_mut().enumerate() {
                            *value = buffer_for_rom[i].into();
                        }
                        true
                    } else {
                        false  
                    }
                },
            Err(_) => false
        }
    }
}


// overloading [] for read/write access
impl<T: PrimInt + std::convert::From<u8>> std::ops::IndexMut<u16> for Rom<T> {

    fn index_mut(&mut self, index: u16) -> &mut T {
        return {
            if index > 0 && index < self.rom.len() as u16 {
                self.rom.get_mut(index as usize).unwrap()
            } else {
                panic!("Invalid write Address"); 
            }
        };
        
    }
}

// helper functions
impl<T: PrimInt + std::convert::From<u8>> Rom<T> {
    pub fn len(&self) -> usize {
        self.rom.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

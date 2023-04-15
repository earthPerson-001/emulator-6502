use std::io::{BufReader, Read, ErrorKind};
use std::fs::File;

use num::traits::int::PrimInt;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
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

impl<T: PrimInt + std::convert::From<u8>> From<Vec<T>> for Rom<T> {
    fn from(vector: Vec<T>) -> Self {
        Self {
            rom: vector,
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

    pub fn load(&mut self, filepath: &str, start_location: &u16) -> bool {


        let file = File::open(filepath);

        let total_bytes_to_read: i32 = self.rom.len() as i32 - *start_location as i32;
        
        let buffer_length =  if total_bytes_to_read < 1 {
            0
        } else {
            total_bytes_to_read as usize
        };


        // to place the read file
        let mut buffer_for_rom = vec![0 as u8; buffer_length];

        match file {
            Ok(opened_file) => {
                    let mut buffered_reader = BufReader::new(opened_file);
                    let buffer_read_result = buffered_reader.read_exact(&mut buffer_for_rom[0..0 + buffer_length]);

                    match buffer_read_result {
                        Ok(_) => {
                            // copying the value from buffer to rom
                            for i in (*start_location as usize)..(*start_location as usize) + buffer_length {
                                self.rom[i] = buffer_for_rom[i].into();
                            }
                            true
                        }
                        ,
                        Err(err) => {
                            match err.kind() {
                                // if eof is reached before filling the buffer
                                // we can safely copy the rest of the buffer, as it was initialized with zeros
                                ErrorKind::UnexpectedEof => {
                                    // copying the value from buffer to rom
                                    for i in (*start_location as usize)..(*start_location as usize) + buffer_length {
                                        self.rom[i] = buffer_for_rom[i].into();
                                    }
                                    true
                                },
                                _ => false,
                            }
                        },
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
            if index < self.rom.len() as u16 {
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

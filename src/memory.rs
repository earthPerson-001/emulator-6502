use num::traits::int::PrimInt;

pub struct Memory<T: PrimInt + std::convert::From<u8>> {
    mem: Vec<T>,
}

impl<T: PrimInt + std::convert::From<u8>> Memory<T> {
    /**
    Returns the memory with given size

    Creates a vector filling the given size with zeros

    # Arguments

    * `size_b` - The memory size in bytes

    */
    pub fn new(size_b: usize) -> Self {
        Self {
            mem: vec![T::zero(); size_b],
        }
    }
}

// overloading [] for read access
impl<T: PrimInt + std::convert::From<u8>> std::ops::Index<u16> for Memory<T> {
    type Output = T;

    fn index(&self, index: u16) -> &T {
        return {
            if index < self.mem.len() as u16 {
                let opt = self.mem.get(index as usize);
                match opt {
                    Some(val) => val,
                    None => panic!("Invalid Read Address"),
                }
            } else {
                panic!("Invalid Read Address")
            }
        };
    }
}

// overloading [] for read/write access
impl<T: PrimInt + std::convert::From<u8>> std::ops::IndexMut<u16> for Memory<T> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        return {
            if index < self.mem.len() as u16 {
                self.mem.get_mut(index as usize).unwrap()
            } else {
                panic!("Invalid write Address"); 
            }
        };
        
    }
}

// helper functions
impl<T: PrimInt + std::convert::From<u8>> Memory<T> {
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

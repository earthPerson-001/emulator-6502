pub struct Memory<T> {
    mem: Vec<T>,
}

impl Memory<u8> {
    /**
    Returns the memory with given size

    Creates a vector filling the given size with zeros

    # Arguments

    * `size_b` - The memory size in bytes

    */
    pub fn new(size_b: usize) -> Self {
        Self {
            mem: vec![0; size_b],
        }
    }
}

// overloading [] for read access
impl std::ops::Index<u16> for Memory<u8> {
    type Output = u8;

    fn index(&self, index: u16) -> &u8 {
        return {
            if index > 0 && index < self.mem.len() as u16 {
                let opt = self.mem.get(index as usize);
                match opt {
                    Some(val) => val,
                    None => &0,
                }
            } else {
                &0
            }
        };
    }
}

// overloading [] for read/write access
impl std::ops::IndexMut<u16> for Memory<u8> {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        return {
            if index > 0 && index < self.mem.len() as u16 {
                self.mem.get_mut(index as usize).unwrap()
            } else {
                panic!("Invalid write Address"); 
            }
        };
        
    }
}

// helper functions
impl Memory<u8> {
    pub fn len(&self) -> usize {
        self.mem.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

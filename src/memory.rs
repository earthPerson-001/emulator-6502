
pub struct Memory {
    mem: Vec<u8>,
}

impl Memory {

    /**
    Returns the memory with given size

    Creates a vector filling the given size with zeros

    # Arguments

    * `size_b` - The memory size in bytes 

    */
    pub fn new(size_b: usize) -> Self {
        Self {
            mem: vec![0; size_b]
        }
    }
}


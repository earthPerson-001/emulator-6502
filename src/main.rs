pub mod bus;
pub mod memory;
pub mod processor;

use crate::processor::Instruction;

fn display_instruction_set() -> () {
    let vec = Instruction::create_instructions_table();
    println!("length of vector: {}", vec.len());

    for (i, instruction) in vec.iter().enumerate() {
        print!("{}", instruction);

        if (i + 1) % 16 == 0 {
            println!("");
        }
    }

}

fn main() -> () {
    #[cfg(debug_assertions)] 
    {
        display_instruction_set();
    }

    println!("Hello World!");
}

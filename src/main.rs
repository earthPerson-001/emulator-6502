use emulator_6502::Processor;

pub mod bus;
pub mod memory;
pub mod processor;
pub mod rom;

use std::env;

#[cfg(debug_assertions)]
fn display_instruction_set() -> () {
    use crate::processor::Instruction;

    let vec = Instruction::create_instructions_table();
    println!("length of vector: {}", vec.len());

    print!("\n{:8}", " ");
    for i in 0..16 {
        print!("{:^12}", format!("{:01x}", i));
    }

    for (i, instruction) in vec.iter().enumerate() {
        if (i == 0) || i % 16 == 0 {
            print!("{:^12}", format!("\n {:01x}", (i + 1)/16));
        }

        print!("{}", instruction);
    }

}

fn main() -> () {
    #[cfg(debug_assertions)] 
    {
        display_instruction_set();
    }

    // new processor instance
    let mut proc = Processor::new();
    
    let mut current_dir = String::from("");
    match env::current_dir() {
        Ok(current_dir_temp) => { 
            current_dir = current_dir_temp.to_str().unwrap().to_owned();
        },
        Err(_) => {
            current_dir = String::from("None");
        },
    }

    let file_path = current_dir +  "/6502_functional_test.bin";
    // loading the rom file
    if proc.load_rom(file_path.as_str()) {
        println!("Read File {} success", file_path);

        let max_cycles = 10000000;

        // running the cpu
        let mut cycle_count = 0;
        loop {
            proc.clock();

            if cycle_count > max_cycles {
                break
            }
            cycle_count += 1;
        }
        println!("Program Complete");

    } else {
        println!("Read File {} failed", file_path);
    }

}

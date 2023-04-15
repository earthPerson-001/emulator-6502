use emulator_6502::Processor;

pub mod bus;
pub mod memory;
pub mod processor;
pub mod rom;

use std::{env, path::Path, time::Duration};

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
            print!("{:^12}", format!("\n {:01x}", (i + 1) / 16));
        }

        print!("{}", instruction);
    }
}

fn main() -> () {
    #[cfg(debug_assertions)]
    {
        display_instruction_set();
    }

    const PROGRAM_COUNTER: u16 = 0x8000;

    // new processor instance
    let mut proc = Processor::new_setup(Some(PROGRAM_COUNTER));

    let current_dir = match env::current_dir() {
        Ok(temp) => Some(Path::new(&temp).to_owned()),
        Err(_) => None,
    };
    let rom_name = r"rom_1";

    let full_rom_path = match current_dir {
        Some(dir) => {
            Some(dir.join("roms").join(format!("{rom_name}.bin")))
        }
        None => None
    };

    if let Some(rom_path) = &full_rom_path {
        // loading the rom file
        if let Some(rom_path) = rom_path.to_str() {
            #[cfg(debug_assertions)]
            {
                println!("\nTrying to read rom from path {:?}", rom_path)
            }

            if proc.load_rom(rom_path, &PROGRAM_COUNTER) {
                println!("Read File {} success", rom_path);

                let max_cycles = 10000000;

                // running the cpu
                let mut cycle_count = 0;
                loop {
                    let anything: Vec<String> = proc.disassembly(&proc.program_counter, &10)
                        .iter()
                        .map(|(_, human_readable)| format!("{}", human_readable))
                        .collect();

                    proc.clock();

                    if cycle_count > max_cycles {
                        break;
                    }
                    cycle_count += 1;

                    (1..20).into_iter().map(|_| print!("-")).for_each(drop);
                    println!("");
                    for thing in anything {
                        println!("{}", thing);
                    }
                    (1..20).into_iter().map(|_| print!("-")).for_each(drop);

                    // wait for 1 sec
                    std::thread::sleep(Duration::new(1, 0))
                }
                println!("Program Complete");
                return;
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        println!("\nRead ROM File {:?} failed", &full_rom_path);
    }
}

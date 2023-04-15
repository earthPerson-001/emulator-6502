mod bus;
mod memory;
mod processor;
mod rom;

pub use bus::Bus;
pub use memory::Memory;
pub use processor::Processor;

use wasm_bindgen::prelude::*;

use serde_json;
use std::{cell::RefCell, collections::HashMap, u8};

#[wasm_bindgen]
extern "C" {
    /**
     * The alert function in JS
     */
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    /**
     * The console.log function in JS
     */
    fn log(s: &str);
}

struct Instance {
    processor: Option<Processor>,
    total_clock_cycle: u16,
}

/*
 * Making a processor instance global variable as to access it from outside.
 * As JS is single threaded, this won't be a problem
 */
thread_local! (
    static INSTANCE: RefCell<Instance> = RefCell::new( Instance{processor: None, total_clock_cycle: 0,} )
);

#[wasm_bindgen(js_name = createProcessor)]
/**
 * Creates new Processor with default values.
 */
pub fn create_processor(program_counter: Option<u16>) {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();

        instance.processor = Some(Processor::new_setup(program_counter));
        instance.total_clock_cycle = 0;
    });

    log("An instance of processor created");
}

#[wasm_bindgen(js_name=clearProcessorInstance)]
/**
 * Clears the current processor instance.
 */
pub fn clear_processor_instance() {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();

        instance.processor = None;
        instance.total_clock_cycle = 0;
    });

    log("An instance of processor created");
}

#[wasm_bindgen(js_name=tickClock)]
/**
 * Performs one fecth, decode and execute cycle.
 */
pub fn tick_clock() {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                proc.reset();
                instance.total_clock_cycle += 1;
                log("Called tick_clock through wasm");
            }
            None => (),
        }
    })
}

#[wasm_bindgen(js_name=loadRomFromFilepath)]
/**
 * Load the rom contents from the given file
 */
pub fn load_rom_from_filepath(filepath: &str, start_location: u16) {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                proc.load_rom(filepath, &start_location);
            }
            None => (),
        }
    })
}

#[wasm_bindgen(js_name=getRam)]
/**
 * Returns serialized memroy.
 * Upon accessing the field ram, the underlying array of u8 is obtained
 */
pub fn get_ram() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => serde_json::to_string(&proc.bus.memory).unwrap(),
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getRom)]
/**
 * Returns serialized secondary_storage.
 * Upon accessing the field rom, the underlying array of u8 is obtained
 */
pub fn get_rom() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => serde_json::to_string(&proc.bus.secondary_storage).unwrap(),
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getProcessorStatus)]
/**
 * Returns a byte representing processor status bits
 */
pub fn get_processor_status() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => serde_json::to_string(&proc.status).unwrap(),
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=loadRom)]
/**
 * Assumes that the bits given are in hexadecimal format and are in order.
 * Also, a byte is represented by 2 hexadecimal bits.
 */
pub fn load_rom(bytes: String, start_location: Option<u16>) -> bool {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                // Spilliting into the groups of 2 as two hexadecimal digits = 8 bits
                let chars: Vec<char> = bytes.chars().collect();
                let split = &chars
                    .chunks(2)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect::<Vec<_>>();

                let nums = split
                    .into_iter()
                    .map(|ch| u8::from_str_radix(ch, 16).expect("Conversion Error"))
                    .collect::<Vec<u8>>();

                log(format!("Loaded {:?}", nums).as_str());

                match start_location {
                    Some(location) => {
                        // changing the rom
                        for (i, val) in nums.iter().enumerate() {
                            if i < proc.bus.secondary_storage.len() {
                                proc.bus.write(location as u16 + i as u16, *val);
                            }
                        }
                    }
                    None => {
                        // changing the rom
                        for (i, val) in nums.iter().enumerate() {
                            if i < proc.bus.secondary_storage.len() {
                                proc.bus.write(i as u16, *val);
                            }
                        }
                    }
                };
                true
            }
            None => false
        }
    })
}

#[wasm_bindgen(js_name=getStorageLayout)]
/**
*   Returns serialized HashMap of [ storage : (start_index, end_index) ]
* Upon deserialization a Object of {storage : (start_index, end_index)}
   is obtained
*/
pub fn get_storage_layout() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                // suppose there are only three fields in struct bus namely memory, other, and secondary_storage

                let memory_len: usize = proc.bus.memory.len();
                let other_len: usize = proc.bus.other.len();
                let secondary_storage_len: usize = proc.bus.secondary_storage.len();

                let storage_to_location: HashMap<String, (usize, usize)> = HashMap::from([
                    (String::from("memory"), (0, memory_len)),
                    (
                        String::from("stack"),
                        (
                            processor::STACK_ADDRESS_RANGE.0 as usize,
                            processor::STACK_ADDRESS_RANGE.1 as usize,
                        ),
                    ),
                    (String::from("other"), (memory_len, memory_len + other_len)),
                    (
                        String::from("secondary_storage"),
                        (
                            memory_len + other_len,
                            memory_len + other_len + secondary_storage_len,
                        ),
                    ),
                ]);

                serde_json::to_string(&storage_to_location).unwrap()
            }
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getStack)]
/**
 Returns the stack contents
*/
pub fn get_stack() -> std::string::String {
    let (stack_start, stack_end) = processor::STACK_ADDRESS_RANGE;

    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                let stack_contents = &proc.bus.memory[stack_start..=stack_end];

                serde_json::to_string(&stack_contents).unwrap()
            }
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getDisassembly)]
/**
* Gets the disassembly starting from
 the location pointed by program counter returning total of `n_lines`
*/
pub fn get_disassembly(n_lines: u16) -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                serde_json::to_string(&proc.disassembly(&proc.program_counter, &n_lines)).unwrap()
            }
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getDisassemblyRange)]
/**
* Gets the disassembly starting from
 the location pointed by program counter returning total of `n_lines`
*/
pub async fn get_disassembly_range(start: u16, n_lines: u16) -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                serde_json::to_string(&proc.disassembly(&start, &n_lines)).unwrap()
            }
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getDisassemblyAll)]
/**
* Gets the disassembly starting from 0000H TO FFFFH
*/
pub fn get_disassembly_all() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => serde_json::to_string(&proc.disassembly(&0x0000, &0xFFFF)).unwrap(),
            None => "{}".to_owned(),
        }
    })
}

#[wasm_bindgen(js_name=getDefaultProgramCounter)]
/**
 * Returns the default program counter used by processor
 *
 */
pub fn get_default_program_counter() -> u16 {
    processor::INITIAL_PROGRAM_COUNTER
}

#[wasm_bindgen(js_name=getCurrentProgramCounter)]
pub fn get_current_program_counter() -> Option<u16> {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => Some(proc.program_counter),
            None => None,
        }
    })
}

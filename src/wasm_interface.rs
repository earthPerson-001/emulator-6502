mod bus;
mod memory;
mod processor;
mod rom;

pub use bus::Bus;
pub use memory::Memory;
pub use processor::Processor;

use wasm_bindgen::prelude::*;

use serde_json;
use std::cell::RefCell;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

struct Instance {
    processor: Option<Processor>,
    total_clock_cycle: u16,
}

/*
 * As JS is single threaded, this won't be a problem
 */
thread_local! (
    static INSTANCE: RefCell<Instance> = RefCell::new( Instance{processor: None, total_clock_cycle: 0,} )
);

#[wasm_bindgen(js_name = createProcessor)]
// creating processor
pub fn create_processor() {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();

        instance.processor = Some(Processor::new());
        instance.total_clock_cycle = 0;
    });

    log("An instance of processor created");
}

#[wasm_bindgen(js_name=clearProcessorInstance)]
// clearing processor
pub fn clear_processor_instance() {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();

        instance.processor = None;
        instance.total_clock_cycle = 0;
    });

    log("An instance of processor created");
}

#[wasm_bindgen(js_name=tickClock)]
// the clock cycle
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
// load the rom file
pub fn load_rom_from_filepath(filepath: &str) {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
            Some(proc) => {
                proc.load_rom(filepath);
            }
            None => (),
        }
    })
}

#[wasm_bindgen(js_name=getRam)]
pub fn get_ram() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
        Some(proc) => serde_json::to_string(&proc.bus.memory).unwrap(),
        None => "{}".to_owned(),
    }})
}

#[wasm_bindgen(js_name=getRom)]
pub fn get_rom() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
        Some(proc) => serde_json::to_string(&proc.bus.secondary_storage).unwrap(),
        None => "{}".to_owned(),
    }})
}

#[wasm_bindgen(js_name=getProcessorStatus)]
pub fn get_processor_status() -> std::string::String {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
        Some(proc) => serde_json::to_string(&proc.status).unwrap(),
        None => "{}".to_owned(),
    }})
}

#[wasm_bindgen(js_name=loadRom)]
pub fn load_rom(bytes: String) {
    INSTANCE.with(|ins| {
        let mut instance = ins.borrow_mut();
        match &mut instance.processor {
        Some(proc) => {
            // converting string to hexadecimal
            let nums: Vec<u8> = bytes
                .chars()
                .map(|c| c.to_digit(16).expect("Conversion Error") as u8)
                .collect();

            log(format!("Loaded {:?}", nums).as_str());

            // changing the rom
            for (i, val) in nums.iter().enumerate() {
                if i < proc.bus.secondary_storage.len() {
                    proc.bus.secondary_storage[i as u16] = *val;
                }
            }
        }
        None => {}
    }})
}

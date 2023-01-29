mod bus;
mod memory;
mod processor;
mod rom;

pub use bus::Bus;
pub use memory::Memory;
pub use processor::Processor;

use wasm_bindgen::prelude::*;

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

// An instance for a page
static mut INSTANCE: Instance = Instance {
    processor: None,
    total_clock_cycle: 0,
};

#[wasm_bindgen]
// creating processor
// unsafe method for now
pub unsafe fn create_processor() {
    INSTANCE.processor = Some(Processor::new());
    INSTANCE.total_clock_cycle = 0;

    log("An instance of processor created");
}

#[wasm_bindgen]
// the clock cycle
// unsafe method for now
pub unsafe fn tick_clock() {
    match &mut INSTANCE.processor {
        Some(proc) => {
            proc.clock();
            INSTANCE.total_clock_cycle += 1;
            log("Called tick_clock through wasm");
        },
        None => (),
    }
}

#[wasm_bindgen]
// load the rom file
// unsafe method for now
pub unsafe fn load_rom(filepath: &str) {
    match &mut INSTANCE.processor {
        Some(proc) => {
            proc.load_rom(filepath);
        },
        None => (),
    }
}

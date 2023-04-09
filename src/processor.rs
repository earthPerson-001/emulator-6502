use crate::bus::Bus;
use crate::memory::Memory;
use crate::rom::Rom;

// Status bits
// representing the number of left shift required to get the bit from 0x01
const CARRY_POS: u8 = 0; // c
const ZERO_POS: u8 = 1; // z
const INTERRUPT_DISABLE_POS: u8 = 2; // i
const DECIMAL_POS: u8 = 3; // d
const B_FLAG_POS: u8 = 4; // b
const UNUSED_FLAG_POS: u8 = 5; // u         //  ignored in some documentations
const OVERFLOW_POS: u8 = 6; // o
const NEGATIVE_POS: u8 = 7; // n

// Sizes
// Setting the size to be 64 KB, in case of 6502
const KB: usize = 1024;

// memory
const RAM: usize = 16 * KB; // 16 KB RAM    (0 - 16 * 1024-1) ( 0x0000 - 0x3FFF)
const STACK_ADDRESS_RANGE: (u16, u16) = (0x0100, 0x01FF);

const OTHER: usize = 16 * KB; // 16 KB other addressed (like 16KB, but the addresses might be other devices) (16* 1024 - 32 * 1024) (0x4000 - 0x7FFF)
const ROM: usize = 32 * KB; // 32 KB ROM (32 * 1024 - 64 * 1024) (0x8000 - 0xFFFF)

// Initial address of program counter
const INITIAL_PROGRAM_COUNTER_ADDRESS: u16 = 0xFFFC;    // 0xFFFC and 0xFFFD are the addresses for initial program counter

// Fixed address to read from when interrupt/ break occurs

/// valid for breaks and irq
const FIXED_READING_ADDRESS_FOR_BRK_AND_IRQ: u16 = 0xFFFE;     // 0xFFFE and 0xFFFF are the address when break or interrupt occurs

/// valid for nmi
const FIXED_READING_ADDRESS_FOR_NMI: u16 = 0xFFFA;

// 6502
/**
* 6502 is little endian, valid for 16 bit addresses
* i.e. $LLHH
*/
pub struct Processor {
    pub bus: Bus<u8>,

    // Instruction set
    instructions: Vec<Instruction>,

    // CPU core registers
    accumulator: u8,
    index_register_x: u8,
    index_register_y: u8,
    pub status: u8,
    /// To be consistent across the implementations
    /// `stack_pointer` should always point to the empty location
    /// * i.e to get the last item, `stack_pointer` should be incremented before reading
    /// * and to add a item, `stack_pointer` should be decremented after writing 
    stack_pointer: u8,
    program_counter: u16,

    // some variables for implementing adressing mode and opcodes
    fetched: u8,
    temp: u16,
    address_absolute: u16,
    address_relative: u16,
    opcode: u8,
    cycles: u8,

    // Variables denoting the stack location in RAM
    // stack (Reversed)
    /// The top of the stack
    stack_last_address: u16,

    #[allow(dead_code)] // might be useful to detect stack overflow later
    stack_first_address: u16
}

impl Default for Processor {
    /**
    * The default value for memory field is memory with given size (bytes)
    * For the instruction set, the default value is the value returned by `create_instruction_table()` function of struct `Instruction`
    * Everything else defaults to zero for integers
    */
    fn default() -> Self {
        Self {
            bus: Bus::new(Memory::new(RAM), vec![0; OTHER], Rom::new(ROM)),
            instructions: Instruction::create_instructions_table(),
            accumulator: 0x00,
            index_register_x: 0x00,
            index_register_y: 0x00,
            status: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,

            fetched: 0x00,
            temp: 0x00,
            address_absolute: 0x0000,
            address_relative: 0x00,
            opcode: 0x00,
            cycles: 0x00,

            stack_last_address: STACK_ADDRESS_RANGE.0,
            stack_first_address: STACK_ADDRESS_RANGE.1, 
        }
    }
}

// Constructor like implementation
// using default values
impl Processor {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

// load rom implementation
impl Processor {
    pub fn load_rom(&mut self, filepath: &str) -> bool {
        self.bus.load_rom(filepath)
    }
}

/*
 * Implementation of getters and setters for status bits
 * here setting means setting it to given boolean value (not the set and reset concept)
 *
 * Single byte implementation for each bit is easier to implement and understand
 * but this is for proper understanding of how 6502 really works.
 * i.e. I know this is overkill.
 */
impl Processor {
    pub fn get_c(&self) -> bool {
        let compare_bit = 0x01 << CARRY_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_z(&self) -> bool {
        let compare_bit = 0x01 << ZERO_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_i(&self) -> bool {
        let compare_bit = 0x01 << INTERRUPT_DISABLE_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_d(&self) -> bool {
        let compare_bit = 0x01 << DECIMAL_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_b(&self) -> bool {
        let compare_bit = 0x01 << B_FLAG_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_u(&self) -> bool {
        let compare_bit = 0x01 << UNUSED_FLAG_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_o(&self) -> bool {
        let compare_bit = 0x01 << OVERFLOW_POS;
        (self.status & compare_bit) != 0
    }

    pub fn get_n(&self) -> bool {
        let compare_bit = 0x01 << NEGATIVE_POS;
        (self.status & compare_bit) != 0
    }

    pub fn set_c(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << CARRY_POS,
            false => self.status &= !(0x1 << CARRY_POS),
        }
    }

    pub fn set_z(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << ZERO_POS,
            false => self.status &= !(0x1 << ZERO_POS),
        }
    }

    pub fn set_i(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << INTERRUPT_DISABLE_POS,
            false => self.status &= !(0x1 << INTERRUPT_DISABLE_POS),
        }
    }

    pub fn set_d(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << DECIMAL_POS,
            false => self.status &= !(0x1 << DECIMAL_POS),
        }
    }

    pub fn set_b(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << B_FLAG_POS,
            false => self.status &= !(0x1 << B_FLAG_POS),
        }
    }

    pub fn set_u(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << UNUSED_FLAG_POS,
            false => self.status &= !(0x1 << UNUSED_FLAG_POS),
        }
    }

    pub fn set_o(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << OVERFLOW_POS,
            false => self.status &= !(0x1 << OVERFLOW_POS),
        }
    }

    pub fn set_n(&mut self, x: bool) {
        match x {
            true => self.status |= 0x1 << NEGATIVE_POS,
            false => self.status &= !(0x1 << NEGATIVE_POS),
        }
    }

    pub fn clear_status(&mut self) {
        self.status = 0x00;
    }
}


/* enums for getting function names
 * This is because i couldn't find a way to get function name from the function pointer
*/

#[derive(Debug, PartialEq, PartialOrd)]
pub enum AddressingMode {
    ABS,  /* absolute */
    ABSX, /* absolute X-indexed */
    ABSY, /* absolute Y-indexed */
    ZPG,  /* zeropage */
    ZPGX, /* zeropage, X-indexed */
    ZPGY, /* zeropage, Y-indexed */
    IND,  /* indirect */
    INDX, /* indirect, X-indexed */
    INDY, /* indirect, Y-indexed */
    IMPL, /* implied */
    REL,  /* relative */
    IMM,  /* immediate */
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Operation {
    ADC, // add with carry
    AND, // and (with accumulator)
    ASL, // arithmetic shift left
    BCC, // branch on carry clear
    BCS, // branch on carry set
    BEQ, // branch on equal (zero set)
    BIT, // bit test
    BMI, // branch on minus (negative set)
    BNE, // branch on not equal (zero clear)
    BPL, // branch on plus (negative clear)
    BRK, // break / interrupt
    BVC, // branch on overflow clear
    BVS, // branch on overflow set
    CLC, // clear carry
    CLD, // clear decimal
    CLI, // clear interrupt disable
    CLV, // clear overflow
    CMP, // compare (with accumulator)
    CPX, // compare with X
    CPY, // compare with Y
    DEC, // decrement
    DEX, // decrement X
    DEY, // decrement Y
    EOR, // exclusive or (with accumulator)
    INC, // increment
    INX, // increment X
    INY, // increment Y
    JMP, // jump
    JSR, // jump subroutine
    LDA, // load accumulator
    LDX, // load X
    LDY, // load Y
    LSR, // logical shift right
    NOP, // no operation
    ORA, // or with accumulator
    PHA, // push accumulator
    PHP, // push processor status (SR)
    PLA, // pull accumulator
    PLP, // pull processor status (SR)
    ROL, // rotate left
    ROR, // rotate right
    RTI, // return from interrupt
    RTS, // return from subroutine
    SBC, // subtract with carry
    SEC, // set carry
    SED, // set decimal
    SEI, // set interrupt disable
    STA, // store accumulator
    STX, // store X
    STY, // store Y
    TAX, // transfer accumulator to X
    TAY, // transfer accumulator to Y
    TSX, // transfer stack pointer to X
    TXA, // transfer X to accumulator
    TXS, // transfer X to stack pointer
    TYA, // transfer Y to accumulator

    // illegal opcodes
    SLO,// ASL oper + ORA oper
    JAM,// Freeze the CPU
    ANC,// AND oper + set C as ASL
    RLA,// ROL oper + AND oper
    SRE,//(LSE) LSR oper + EOR oper
    ALR,// (ASR) AND oper + LSR
    RRA, // ROR oper + ADC oper
    SAX, 
    ANE, 
    SHA, 
    SHX, 
    SHY, 
    ARR, 
    TAS, 
    LAS, 
    LAX, 
    LXA, 
    DCP, 
    SBX, 
    ISC, 
    USBC,
}

// reset function implementation
impl Processor {

    /**
     * Forces the CPU into known state
     * 
     # Operations
     * Set the program counter to the address stored at `0xFFFD` | `0xFFFC` 
     * Reset the internal registers 
     */
    pub fn reset(&mut self) -> () {
        // set the next address for program counter
        self.program_counter = self.bus.read(INITIAL_PROGRAM_COUNTER_ADDRESS + 1 as u16) as u16 
                                | self.bus.read(INITIAL_PROGRAM_COUNTER_ADDRESS) as u16;
        
        // reset internal registers
        self.accumulator = 0x00;
        self.index_register_x = 0x00;
        self.index_register_y = 0x00;
        self.stack_pointer = 0xFF;
        self.status = 0x00;

        self.set_u(true); // always setting the unused state to true

        // clearing helper variables
        self.fetched = 0x00;
        self.address_absolute = 0x0000;
        self.address_relative = 0x0000;

        self.cycles = 8; // reset takes time

    }
}


// Interrupts implementations 
impl Processor {

    // can be ignored
    fn irq(&mut self) {
        // if interrupts are allowed // it might not be allowed when interrupt is ongoing
        if !self.get_i() {

            // pushing the current program counter to stack
            self.bus.write(self.stack_last_address + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8 );
            self.stack_pointer -= 1;
            self.bus.write(self.stack_last_address + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8 );
            self.stack_pointer -= 1;

            // changing the status registers
            self.set_b(false); self.set_u(true); self.set_i(true);

            // pushing the processor status to stack
            self.bus.write(self.stack_last_address + self.stack_pointer as u16, self.status);
            self.stack_pointer -= 1;

            // reading the new program counter from the fixed address
            self.program_counter = (self.bus.read(FIXED_READING_ADDRESS_FOR_BRK_AND_IRQ + 1) as u16) << 8 
                                    | (self.bus.read(FIXED_READING_ADDRESS_FOR_BRK_AND_IRQ)) as u16;

            // some time is required for irq
            self.cycles = 7;
        }
    }

    // cannot be ignored
    fn nmi(&mut self) {
        // pushing the current program counter to stack
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8 );
        self.stack_pointer -= 1;
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8 );
        self.stack_pointer -= 1;

        // changing the status registers
        self.set_b(false); self.set_u(true); self.set_i(true);

        // pushing the processor status to stack
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, self.status);
        self.stack_pointer -= 1;

        // reading the new program counter from the fixed address
        self.program_counter = (self.bus.read(FIXED_READING_ADDRESS_FOR_NMI + 1) as u16) << 8 
                                | (self.bus.read(FIXED_READING_ADDRESS_FOR_NMI)) as u16;

        // some time is required for nmi
        self.cycles = 8;
    }

} 

// CPU clock implementation
// typical fetch, decode, execute cycle
impl Processor {

    pub fn clock(&mut self) {
        /*
            The instruction set is stored in such a way that it's index corresponds to the opcode.
            Since, hex and decimal number are equivalent,
            it doesn't matter if the opcode is represented in hex when storing in ROM or any other storage
        */

        // if there are no other pending instruction (previous instruction's execution has completed)
        if self.cycles == 0  {

            // the next instruction byte (aka opcode)
            self.opcode = self.bus.read(self.program_counter);

            // always set the unused falg to 1 
            self.set_u(true);

            // incrementing the program counter as this instruction is already read
            // and instruction may not execute next one immediately ( turns out this is a standard practice)
            // i.e fetch instruction -> increment program counter -> execute instruction
            self.program_counter += 1;

            // get the starting number of cycles
            self.cycles = self.instructions.get(self.opcode as usize).unwrap().cycles;

            // performing the fetch operation
            // and finding out if additional cycle is required by fetch
            let additional_cycle_for_fetch = (self.instructions.get(self.opcode as usize).unwrap().addressing_mode)(self);
            // performing the execute operation 
            // and finding out if the operation has the potential to require additional cycle
            let additional_cycle_for_execute = (self.instructions.get(self.opcode as usize).unwrap().addressing_mode)(self);

            // if more additional cycle is required by particular operation
            // then it should be incremented inside of the operation
            
            // incrementing cycle if the fetch operation required more cycle and execute operation had the potential to require more cycle
            self.cycles += (additional_cycle_for_fetch && additional_cycle_for_execute) as u8;

            // always set the unused falg to 1 
            self.set_u(true);
        }

        if self.cycles == 0 {
            return;
        }
        // decrementing the required cycle for the currently running instruction
        // as one cycle has passed
        self.cycles -= 1;

    }
}

/**
 * Addressing modes implementation
 *
 * These implementations make sure that the operand is available for a particular operation.
 * i.e. the instruction should utilize an addressing mode and an operation
 *
 * Addressing modes will return false for general cases and true for exceptions or unusual cases
 */
#[allow(non_snake_case)]
impl Processor {
    /**
     # Description
    * Absolute
    * The data is present in
      16 bit address present in program counter in the form of little endian $LLHH
    */
    fn ABS(&mut self) -> bool {
        self.address_absolute = (self.bus.read(self.program_counter + 1) as u16) << 8
            | self.bus.read(self.program_counter) as u16;
        self.program_counter += 2;

        false
    }

    /**
    * Returns `false` if page doesn't change `true` otherwise
    # Description
    *  Absolute X
    *  Same as `ABS()` but the value of  `index_register_x` is added to the address and
       if the page changes, then one more cycle is required so boolean value of `true` is returned
    */
    fn ABSX(&mut self) -> bool {
        self.address_absolute = (self.bus.read(self.program_counter + 1) as u16) << 8
            | self.bus.read(self.program_counter) as u16;
        self.program_counter += 2;

        let old_high_bits = 0xFF00 & self.address_absolute;

        self.address_absolute += self.index_register_x as u16;

        let new_high_bits = 0xFF00 & self.address_absolute;

        !(old_high_bits == new_high_bits)
    }

    /**
    * Returns `false` if page doesn't change `true` otherwise
    # Description
    *  Absolute Y
    *  Same as `ABS()` but the value of  `index_register_y` is added to the address and
       if the page changes, then one more cycle is required so boolean value of `true` is returned
    */
    fn ABSY(&mut self) -> bool {
        self.address_absolute = (self.bus.read(self.program_counter + 1) as u16) << 8
            | self.bus.read(self.program_counter) as u16;
        self.program_counter += 2;

        let old_high_bits = 0xFF00 & self.address_absolute;

        self.address_absolute += self.index_register_y as u16;

        let new_high_bits = 0xFF00 & self.address_absolute;

        !(old_high_bits == new_high_bits)
    }

    /**
    # Description
    * zeropage

    * The address of the data is at the program counter address
    * Data present at 0x00 - 0xFF
    */
    fn ZPG(&mut self) -> bool {
        self.address_absolute = 0x00FF & self.bus.read(self.program_counter) as u16;
        self.program_counter += 1;

        false
    }

    /**
     *  zeropage, X-indexed
     * Same as `ZPG()` but the address is present at offset `index_register_x` from `program_counter`
     */
    fn ZPGX(&mut self) -> bool {
        self.address_absolute = 0x00FF
            & self
                .bus
                .read(self.program_counter + self.index_register_x as u16) as u16;
        self.program_counter += 1;

        false
    }

    /**
    # Description
    *  zeropage, Y-indexed
    * Same as `ZPG()` but the address is present at offset `index_register_y` from `program_counter`
    */
    fn ZPGY(&mut self) -> bool {
        self.address_absolute = 0x00FF
            & self
                .bus
                .read(self.program_counter + self.index_register_y as u16) as u16;
        self.program_counter += 1;

        false
    }

    /**
     # Description
    *  Indirect
    *  In indirect modes, the provided 16 bit address is used to lookup the actual 16 bit address
       . In a sense, this behaves like address pointers

    * There is a hardware bug in this mode, and we need to emulate that too
    * */
    fn IND(&mut self) -> bool {
        let pointer_low = self.bus.read(self.program_counter) as u16;
        self.program_counter += 1;

        let pointer_high = self.bus.read(self.program_counter) as u16;
        self.program_counter += 1;

        let pointer = pointer_high << 8 | pointer_low;

        // The high bits will be read from the start of the same page because of the hardware bug

        if pointer_low == 0x00FF {
            // Simulate the page boundary hardware bug
            self.address_absolute =
                (self.bus.read(pointer_high) as u16) << 8 | self.bus.read(pointer + 0) as u16;
        } else {
            // behave normally
            self.address_absolute =
                ((self.bus.read(pointer + 1)) as u16) << 8 | self.bus.read(pointer + 0) as u16;
        }

        false
    }

    /**
     # Description
    *  indirect, X-indexed, also utilizes zero page

    * The supplied 8 bit address is offset by the value in `index_register_x` to index a location in zero-page
    * and the actual address is read from the given address and the consequent one
    */
    fn INDX(&mut self) -> bool {
        let actual_pointer = (self.bus.read(self.program_counter) + self.index_register_x) as u16;
        self.program_counter += 1;

        self.address_absolute = ((self.bus.read(actual_pointer + 1) as u16) << 8)
            | (self.bus.read(actual_pointer) as u16) << 8;

        false
    }

    /**
    # Returns
    false if offset causes page change and additional cycle is required
    true otherwise

    # Description
    *  indirect, Y-indexed, also utilizes zero page

    * The supplied 8 bit address is used to lookup another address which is offset by the content of `index_register_y`
     to get the final address

    * Here, first a pair of 8-bit addresses is found in zero-page to make the 16 bit address
     which is then offset by the value in `index_register_y` to get the final address

    * If the addition of offset causes page change, then additional clock cycle is required
    */
    fn INDY(&mut self) -> bool {
        let address_before_offset = (self
            .bus
            .read((self.bus.read((self.program_counter) as u16) + 1) as u16)
            as u16)
            << 8
            | (self
                .bus
                .read(self.bus.read(self.program_counter as u16) as u16)) as u16;
        self.program_counter += 1;

        self.address_absolute = address_before_offset + self.index_register_y as u16;

        address_before_offset & 0xFF00 != self.address_absolute & 0xFF00
    }

    /**
    # Description
     * implied
     * No additional data required
     */
    fn IMPL(&mut self) -> bool {
        self.fetched = self.accumulator;

        false
    }

    /**
    # Description
     *  relative */
    fn REL(&mut self) -> bool {
        self.address_relative = self.bus.read(self.program_counter) as u16;
        self.program_counter += 1;

        // if the relative address is negative
        if (self.address_relative & 0x80) == 0x80 {
            self.address_relative |= 0xFF00;    // make every other bit 1 (denoting negative address)
        }

        false
    }

    /** immediate
     * Data present on the next address of the opcode
     */
    fn IMM(&mut self) -> bool {
        self.address_absolute = self.program_counter + 1;

        false
    }
}

impl Processor {
    /**
     * changes `self.fetched` to the fetched value
     # Returns
     the fetched value

    # Description
    * Convenience method for fetching before carrying out the operation
    * Fetches either from accumulator or from the memory
    */
    fn fetch(&mut self) -> u8 {
        if self.instructions[self.opcode as usize].addressing_mode_enum  == AddressingMode::IMPL { 
            // if the data is present in the accumulator (i.e. in implied addressing mode)
            self.fetched = self.accumulator; 
        } else {
            self.fetched = self.bus.read(self.address_absolute);
        }
        self.fetched
    }
}

/*
 * OP Codes implementation
 */
#[allow(non_snake_case)]
impl Processor {
    // add with carry
    fn ADC(&mut self) -> bool {
        self.fetch();

        self.temp = self.accumulator as u16 + self.fetched as u16 + self.get_c() as u16;

        // carry flag is set if the sum of two 8 bit number takes 9 bit
        self.set_c(self.temp > 255);

        // zero flag is set if the number is zero
        self.set_z(self.temp & 0x00FF == 0x0000); // checking only 8 bits

        // finding out if it has overflowed
        // and setting it as overflow flag
        let sign_bit = 1 << 7;
        self.set_o(
            (((self.accumulator & sign_bit) == 0)       // if the result is negative given both the operands are positive
                && (self.fetched & sign_bit == 0)
                && (self.temp & sign_bit as u16 == 1))
                || (((self.accumulator & sign_bit) == 1)    // if the result is positive given both the operands are negative
                    && (self.fetched & sign_bit == 1)
                    && (self.temp & sign_bit as u16 == 0)),
        );

        // setting the negative flag
        self.set_n(self.temp & sign_bit as u16 == 1);

        // loading the result into accumulator (the part except carry, if any)
        self.accumulator = (self.temp & 0x00FF) as u8;

        // has the potential to require additional clock cycle
        true
    }

    // and (with accumulator)
    fn AND(&mut self) -> bool {

        // bitwise and with the value in memory
        self.accumulator &= self.fetch();

        // zero flag
        self.set_z(self.accumulator == 0x00);

        // negative flag
        self.set_n(self.accumulator & (1 << 7) == 0x01 );

        true
    }

    // arithmetic shift left
    fn ASL(&mut self) -> bool {
        let left_shifted_output = (self.fetch() as u16) << 1;

        if self.instructions[self.opcode as usize].addressing_mode_enum == AddressingMode::IMPL {
            self.accumulator = (left_shifted_output & 0x00FF) as u8;
        } else {
            self.bus.write(self.address_absolute, (left_shifted_output & 0x00FF) as u8)
        }

        // doesn't require additional clock cycle
        false
    }

    // branch on carry clear
    fn BCC(&mut self) -> bool {
        if !self.get_c() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // branch on carry set
    fn BCS(&mut self) -> bool {
        if self.get_c(){
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // branch on equal (zero set)
    fn BEQ(&mut self) -> bool {
        if self.get_z() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    /**
     # Description
     * Test Bits in Memory with Accumulator

     * bits 7 and 6 of operand are transfered to bit 7 and 6 of status register (n,o) negative, overflow; BYTE: `7 6 5 4 3 2 1 0`
     * the zero-flag is set to the result of operand AND accumulator.
    */
    fn BIT(&mut self) -> bool {
        self.fetch();

        self.set_n(self.fetched & (1 << 7) == (1<<7));
        self.set_o(self.fetched & (1 << 6) == (1<<6));

        self.set_z((self.fetched & self.accumulator) == 0x00);

        true
    }

    // branch on minus (negative set)
    fn BMI(&mut self) -> bool {
        if self.get_n() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // branch on not equal (zero clear)
    fn BNE(&mut self) -> bool {
        if !self.get_z() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // branch on plus (negative clear)
    fn BPL(&mut self) -> bool {
        if !self.get_n() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // break / interrupt
    fn BRK(&mut self) -> bool {
        self.program_counter += 1;

        // setting interrupt inhibit flag
        self.set_i(true);

        // pushing the program_counter to stack
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, ((self.program_counter >> 8) & 0x00FF) as u8);
        self.stack_pointer -= 1;
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, (self.program_counter & 0x00FF) as u8);
        self.stack_pointer -= 1;

        // setting the break flag
        self.set_b(true);

        // pushing the status register to stack
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, self.status);
        self.stack_pointer -= 1;

        // clearing the break flag
        self.set_b(false);

        // setting the program counter to the value in final addresses (target addresses for break)
        self.program_counter = (self.bus.read(FIXED_READING_ADDRESS_FOR_BRK_AND_IRQ + 1) as u16) << 8 
                                | self.bus.read(FIXED_READING_ADDRESS_FOR_BRK_AND_IRQ) as u16;

        false
    }

    // branch on overflow clear
    fn BVC(&mut self) -> bool {
        if !self.get_o() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // branch on overflow set
    fn BVS(&mut self) -> bool {
        if self.get_o() {
            // adding one cycle (due to branching)
            self.cycles += 1;

            // branching
            self.address_absolute = self.program_counter + self.address_relative as u16;

            // adding one more cycle if branch occurs to different page
            if self.program_counter & 0xFF00 != self.address_absolute & 0xFF00 {
                self.cycles += 1;
            }

            self.program_counter = self.address_absolute;
        }

        // cycles has already been incremented
        false
    }

    // clear carry
    fn CLC(&mut self) -> bool {
        self.set_c(false);

        false
    }

    // clear decimal
    fn CLD(&mut self) -> bool {
        self.set_d(false);

        false
    }

    // clear interrupt disable
    fn CLI(&mut self) -> bool {
        self.set_i(false);

        false
    }

    // clear overflow
    fn CLV(&mut self) -> bool {
        self.set_o(false);

        false
    }

    // compare (with accumulator)
    fn CMP(&mut self) -> bool {
        self.fetch();

        self.set_c(self.accumulator >= self.fetched); // carry flag if accumulator is greater than or equal to the fetched value
        self.set_z((self.accumulator as u16 - self.fetched as u16) == 0x0000); // zero flag if accumulator and fetched both are same (except the carry bit)
        self.set_n((self.accumulator as u16 - self.fetched as u16) & (1 << 7) == (1 << 7)); // negative flag if the result is negative

        // may require on additional clock cycle
        true
    }

    // compare with X
    fn CPX(&mut self) -> bool {
        self.fetch();

        self.set_c(self.index_register_x >= self.fetched); // carry flag if index_register_x is greater than or equal to the fetched value
        self.set_z((self.index_register_x as u16 - self.fetched as u16) == 0x0000); // zero flag if index_register_x and fetched both are same (except the carry bit)
        self.set_n((self.index_register_x as u16 - self.fetched as u16) & (1 << 7) == (1 << 7)); // negative flag if the result is negative

       false        
    }

    // compare with Y
    fn CPY(&mut self) -> bool {
        self.fetch();

        self.set_c(self.index_register_y >= self.fetched); // carry flag if index_register_y is greater than or equal to the fetched value
        self.set_z((self.index_register_y as u16 - self.fetched as u16) == 0x0000); // zero flag if index_register_y and fetched both are same (except the carry bit)
        self.set_n((self.index_register_y as u16 - self.fetched as u16) & (1 << 7) == (1 << 7)); // negative flag if the result is negative

        false
    }

    // decrement the value at memory location
    fn DEC(&mut self) -> bool {
        self.temp = self.fetch() as u16 - 1 as u16;
        self.bus.write(self.address_absolute, (self.temp & 0x00FF) as u8);

        // setting the flags
        self.set_z((self.temp & 0x00FF) == 0x0000);
        self.set_n((self.temp & 0x80) == 0x80);

        false
    }

    // decrement index_register_x
    fn DEX(&mut self) -> bool {
        self.index_register_x -= 1;
        
        // setting the flags
        self.set_z(self.index_register_x == 0x00);
        self.set_n((self.index_register_x & 0x80) == 0x80);

        false
    }

    // decrement index_register_y
    fn DEY(&mut self) -> bool {
        self.index_register_y -= 1;
        
        // setting the flags
        self.set_z(self.index_register_y == 0x00);
        self.set_n((self.index_register_y & 0x80) == 0x80);

        false
    }

    // exclusive or (with accumulator)
    fn EOR(&mut self) -> bool {
        self.accumulator = self.accumulator ^ self.fetch();

        // setting the flags
        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & (1 << 7)) == (1 << 7));

        true
    }

    // increment
    fn INC(&mut self) -> bool {
        self.temp = self.fetch() as u16 + 1 as u16;
        self.bus.write(self.address_absolute, (self.temp & 0x00FF) as u8);

        // setting the flags
        self.set_z((self.temp & 0x00FF) == 0x0000);
        self.set_n((self.temp & 0x80) == 0x80);

        false
    }

    // increment X
    fn INX(&mut self) -> bool {
        self.index_register_x += 1;
        
        // setting the flags
        self.set_z(self.index_register_x == 0x00);
        self.set_n((self.index_register_x & 0x80) == 0x80);

        false
    }

    // increment Y
    fn INY(&mut self) -> bool {
        self.index_register_y += 1;
        
        // setting the flags
        self.set_z(self.index_register_y == 0x00);
        self.set_n((self.index_register_y & 0x80) == 0x80);

        false
    }

    // jump
    fn JMP(&mut self) -> bool {
        self.program_counter = self.address_absolute;

        false
    }

    // jump subroutine
    fn JSR(&mut self) -> bool {
        true
    }

    // load accumulator
    fn LDA(&mut self) -> bool {
        self.accumulator = self.fetch();

        // setting the flags
        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & 0x80) == 0x80);

        true
    }

    // load X
    fn LDX(&mut self) -> bool {
        self.index_register_x = self.fetch();

        // setting the flags
        self.set_z(self.index_register_x == 0x00);
        self.set_n((self.index_register_x & 0x80) == 0x80);

        true
    }

    // load Y
    fn LDY(&mut self) -> bool {
        self.index_register_y = self.fetch();

        // setting the flags
        self.set_z(self.index_register_y == 0x00);
        self.set_n((self.index_register_y & 0x80) == 0x80);

        true
    }

    // logical shift right
    fn LSR(&mut self) -> bool {

        let right_shifted_output = (self.fetch() as u16) >> 1;

        self.set_c(self.fetched & 0x01 == 0x01 );

        if self.instructions[self.opcode as usize].addressing_mode_enum == AddressingMode::IMPL {
            self.accumulator = (right_shifted_output & 0x00FF) as u8;
        } else {
            self.bus.write(self.address_absolute, (right_shifted_output & 0x00FF) as u8)
        }

        // doesn't require additional clock cycle
        false
    }

    // no operation
    fn NOP(&mut self) -> bool {
        true
    }

    // or with accumulator
    fn ORA(&mut self) -> bool {
        self.accumulator = self.accumulator | self.fetch();

        // setting the flags
        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & (1 << 7)) == (1 << 7));

        true
    }

    // push accumulator to stack
    fn PHA(&mut self) -> bool {
        self.bus.write(self.stack_last_address + self.stack_pointer as u16, self.accumulator);
        self.stack_pointer -= 1;

        false
    }

    // push processor status (`self.status`) to stack
    // Break flag is set to 1 before push
    fn PHP(&mut self) -> bool {

        self.bus.write(self.stack_last_address + self.stack_pointer as u16,
             self.status 
             | ((self.get_u() as u8) << UNUSED_FLAG_POS) 
             | (((self.get_b() as u8) <<  B_FLAG_POS)));

        self.set_b(false);
        self.set_u(false);

        self.stack_pointer -= 1;

        false
    }

    // pull accumulator from stack (pop accumulator off stack)
    fn PLA(&mut self) -> bool {
        self.stack_pointer += 1;

        self.accumulator = self.bus.read(self.stack_last_address + self.stack_pointer as u16);

        // setting the flags depending upon the new accumulator value
        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & (1 << 7)) == (1 << 7));

        false
    }

    // pull processor status (`self.status`) (pop status register off stack)
    fn PLP(&mut self) -> bool {
        self.stack_pointer += 1;

        self.status = self.bus.read(self.stack_last_address + self.stack_pointer as u16);

        self.set_u(true);

        false
    }

    // rotate left
    fn ROL(&mut self) -> bool {
        self.temp = ((self.fetch() as u16) << 1) | self.get_c() as u16;

        // setting the flags
        self.set_c((self.temp & 0xFF00) > 0);
        self.set_z((self.temp & 0x00FF) == 0x0000);
        self.set_n((self.temp & 0x0080) == 0x0080);

        // writing to accumulatro or to the memory
        if self.instructions[self.opcode as usize].addressing_mode_enum == AddressingMode::IMPL {
            self.accumulator = (self.temp & 0x00FF) as u8;
        } else {
            self.bus.write(self.address_absolute, (self.temp & 0x00FF) as u8)
        }

        // no additional clock cycle required
        false
    }

    // rotate right
    fn ROR(&mut self) -> bool {
        
        // shifting one bit to right and setting the carry bit in the leftmost bit
        self.temp = ((self.get_c() as u8) << 7) as u16 | ((self.fetch()) >> 1) as u16; 

        // setting carry flag if the removed bit is 1
        self.set_c((self.fetched & 0x01) == 0x01);

        // setting the zero flag if the remaining bits are 0
        self.set_z((self.temp & 0x00FF) == 0x0000);

        // setting the negative flag if bit 7 is 1
        self.set_n((self.temp & (1 << 7)) == (1 << 7));

        // writing to accumulatro or to the memory
        if self.instructions[self.opcode as usize].addressing_mode_enum == AddressingMode::IMPL {
            self.accumulator = (self.temp & 0x00FF) as u8;
        } else {
            self.bus.write(self.address_absolute, (self.temp & 0x00FF) as u8)
        }

        // no additional clock cycle required
        false
    }

    // return from interrupt
    fn RTI(&mut self) -> bool {
        self.stack_pointer += 1;

        // getting the status from stack
        self.status = self.bus.read(self.stack_last_address + self.stack_pointer as u16);

        // changing the break and unused flags
        self.set_b(!self.get_b());
        self.set_u(!self.get_u());

        self.stack_pointer += 1;

        // getting the program counter from stack
        self.program_counter = (self.bus.read(self.stack_last_address + self.stack_pointer as u16 + 1) as u16) << 8 
                                | self.bus.read(self.stack_last_address + self.stack_pointer as u16) as u16;
        self.stack_pointer += 1;

        false
    }

    // return from subroutine
    fn RTS(&mut self) -> bool {
        self.stack_pointer += 1;

        // getting the program counter from stack
        self.program_counter = (self.bus.read(self.stack_last_address + self.stack_pointer as u16 + 1) as u16) << 8 
                                | self.bus.read(self.stack_last_address + self.stack_pointer as u16) as u16;
        self.stack_pointer += 1;

        self.program_counter += 1;

        false
    }

    // subtract with carry (burrow)
    fn SBC(&mut self) -> bool {

        self.fetch();

        // inverting the fetched value (1's complement)
        let inverted_fetched = !self.fetched;

        // adding with the value in memory (subtraction)
        self.temp = self.accumulator as u16 + inverted_fetched as u16 + self.get_c() as u16;

        // same as add with carry
        // carry flag is set if the sum of two 8 bit number takes 9 bit
        self.set_c(self.temp > 255);

        // zero flag is set if the number is zero
        self.set_z(self.temp & 0x00FF == 0x0000); // checking only 8 bits

        // finding out if it has overflowed
        // and setting it as overflow flag
        let sign_bit = (1 as u8) << 7;
        self.set_o(
            (((self.accumulator & sign_bit) == 0)       // if the result is negative given both the operands are positive
                && (inverted_fetched & sign_bit == 0)
                && (self.temp & sign_bit as u16 == 1))
                || (((self.accumulator & sign_bit) == 1) // if the result is positive given both the operands are negative
                    && (inverted_fetched & sign_bit == 1)
                    && (self.temp & sign_bit as u16 == 0)),
        );

        // setting the negative flag
        self.set_n(self.temp & sign_bit as u16 == 1);

        // loading the result into accumulator (the part except carry, if any)
        self.accumulator = (self.temp & 0x00FF) as u8;

        true
    }

    // set carry
    fn SEC(&mut self) -> bool {
        self.set_c(true);

        false
    }

    // set decimal
    fn SED(&mut self) -> bool {
        self.set_d(true);

        false
    }

    // set interrupt disable
    fn SEI(&mut self) -> bool {
        self.set_i(true);

        false
    }

    // store accumulator at address
    fn STA(&mut self) -> bool {
        self.bus.write(self.address_absolute, self.accumulator);

        false
    }

    // store X at address
    fn STX(&mut self) -> bool {
        self.bus.write(self.address_absolute, self.index_register_x);

        false
    }

    // store Y at address
    fn STY(&mut self) -> bool {
        self.bus.write(self.address_absolute, self.index_register_y);

        false
    }

    // transfer accumulator to X register
    fn TAX(&mut self) -> bool {
        self.index_register_x = self.accumulator;

        self.set_z(self.index_register_x == 0x00);
        self.set_n((self.index_register_x & 0x80) == 0x80);

        false
    }

    // transfer accumulator to Y
    fn TAY(&mut self) -> bool {
        self.index_register_y = self.accumulator;

        self.set_z(self.index_register_y == 0x00);
        self.set_n((self.index_register_y & 0x80) == 0x80);

        false
    }

    // transfer stack pointer to X
    fn TSX(&mut self) -> bool {
        self.index_register_x = self.stack_pointer;

        self.set_z(self.index_register_x == 0x00);
        self.set_n((self.index_register_x & 0x80) == 0x80);

        false
    }

    // transfer X to accumulator
    fn TXA(&mut self) -> bool {
        self.accumulator = self.index_register_x;

        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & 0x80) == 0x80);

        false
    }

    // transfer X to stack pointer
    fn TXS(&mut self) -> bool {
        self.stack_pointer = self.index_register_x;
    
        false    
    }

    // transfer Y to accumulator
    fn TYA(&mut self) -> bool {
        self.accumulator = self.index_register_y;

        self.set_z(self.accumulator == 0x00);
        self.set_n((self.accumulator & 0x80) == 0x80);

        false
    }

    // illegal opcodes

    // ASL oper + ORA oper
    fn SLO(&mut self) -> bool {
        true
    }

    // Freeze the CPU
    fn JAM(&mut self) -> bool {
        true
    }

    // AND oper + set C as ASL
    fn ANC(&mut self) -> bool {
        true
    }

    // ROL oper + AND oper
    fn RLA(&mut self) -> bool {
        true
    }

    // (LSE)
    // LSR oper + EOR oper
    fn SRE(&mut self) -> bool {
        true
    }

    // (ASR)
    // AND oper + LSR
    fn ALR(&mut self) -> bool {
        true
    }

    // ROR oper + ADC oper
    fn RRA(&mut self) -> bool {
        true
    }

    fn SAX(&mut self) -> bool {
        true
    }

    fn ANE(&mut self) -> bool {
        true
    }

    fn SHA(&mut self) -> bool {
        true
    }

    fn SHX(&mut self) -> bool {
        true
    }

    fn SHY(&mut self) -> bool {
        true
    }

    fn ARR(&mut self) -> bool {
        true
    }

    fn TAS(&mut self) -> bool {
        true
    }

    fn LAS(&mut self) -> bool {
        true
    }

    fn LAX(&mut self) -> bool {
        true
    }

    fn LXA(&mut self) -> bool {
        true
    }

    fn DCP(&mut self) -> bool {
        true
    }

    fn SBX(&mut self) -> bool {
        true
    }

    fn ISC(&mut self) -> bool {
        true
    }

    fn USBC(&mut self) -> bool {
        true
    }
}
pub struct Instruction {
    pub name: String,
    pub human_readable_form: String,
    pub operation: fn(&mut Processor) -> bool,
    pub operation_enum: Operation,
    pub addressing_mode: fn(&mut Processor) -> bool,
    pub addressing_mode_enum: AddressingMode,
    pub cycles: u8,
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "|{:4} {:4} {}",
            format!("{:?}",self.operation_enum),
            format!("{:?}", self.addressing_mode_enum),
            self.cycles
        )
    }
}

impl Instruction {
    pub fn new(
        name: &str,
        human_readable_form: &str,
        operation: fn(&mut Processor) -> bool,
        operation_enum: Operation,
        addressing_mode: fn(&mut Processor) -> bool,
        addressing_mode_enum: AddressingMode,
        cycles: u8,
    ) -> Self {
        Self {
            name: String::from(name),
            human_readable_form: String::from(human_readable_form),
            operation,
            operation_enum,
            addressing_mode,
            addressing_mode_enum,
            cycles,
        }
    }

    pub fn create_instructions_table() -> Vec<Instruction> {
        let n_c_nop: u8 = 0;
        let n_c_jam: u8 = 0;

        // creating instruction set
        vec![
            Instruction::new(r#"BRK"#, r#""#, Processor::BRK,  Operation::BRK, Processor::IMPL, AddressingMode::IMPL, 7),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::IMPL, AddressingMode::IMPL, n_c_jam),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPG,  AddressingMode::ZPG, n_c_nop),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"ASL"#, r#""#, Processor::ASL,  Operation::ASL, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"PHP"#, r#""#, Processor::PHP,  Operation::PHP, Processor::IMPL, AddressingMode::IMPL, 3),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"ASL"#, r#""#, Processor::ASL,  Operation::ASL, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ANC"#, r#""#, Processor::ANC,  Operation::ANC, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABS,  AddressingMode::ABS, n_c_nop),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"ASL"#, r#""#, Processor::ASL,  Operation::ASL, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BPL"#, r#""#, Processor::BPL,  Operation::BPL, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::IMPL, AddressingMode::IMPL, n_c_jam),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"ASL"#, r#""#, Processor::ASL,  Operation::ASL, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"CLC"#, r#""#, Processor::CLC,  Operation::CLC, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"ORA"#, r#""#, Processor::ORA,  Operation::ORA, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"ASL"#, r#""#, Processor::ASL,  Operation::ASL, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"SLO"#, r#""#, Processor::SLO,  Operation::SLO, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"JSR"#, r#""#, Processor::JSR,  Operation::JSR, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::IMPL, AddressingMode::IMPL, n_c_jam),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"BIT"#, r#""#, Processor::BIT,  Operation::BIT, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"ROL"#, r#""#, Processor::ROL,  Operation::ROL, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"PLP"#, r#""#, Processor::PLP,  Operation::PLP, Processor::IMPL, AddressingMode::IMPL, 4),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"ROL"#, r#""#, Processor::ROL,  Operation::ROL, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ANC"#, r#""#, Processor::ANC,  Operation::ANC, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"BIT"#, r#""#, Processor::BIT,  Operation::BIT, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"ROL"#, r#""#, Processor::ROL,  Operation::ROL, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BMI"#, r#""#, Processor::BMI,  Operation::BMI, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::IMPL, AddressingMode::IMPL, n_c_jam),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::INDY, AddressingMode::INDY, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"ROL"#, r#""#, Processor::ROL,  Operation::ROL, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"SEC"#, r#""#, Processor::SEC,  Operation::SEC, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"AND"#, r#""#, Processor::AND,  Operation::AND, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"ROL"#, r#""#, Processor::ROL,  Operation::ROL, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"RLA"#, r#""#, Processor::RLA,  Operation::RLA, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"RTI"#, r#""#, Processor::RTI,  Operation::RTI, Processor::IMPL, AddressingMode::IMPL, 6),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::IMPL, AddressingMode::IMPL, n_c_jam),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPG,  AddressingMode::ZPG, n_c_nop),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"LSR"#, r#""#, Processor::LSR,  Operation::LSR, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"PHA"#, r#""#, Processor::PHA,  Operation::PHA, Processor::IMPL, AddressingMode::IMPL, 3),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"LSR"#, r#""#, Processor::LSR,  Operation::LSR, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ALR"#, r#""#, Processor::ALR,  Operation::ALR, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"JMP"#, r#""#, Processor::JMP,  Operation::JMP, Processor::ABS,  AddressingMode::ABS, 3),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"LSR"#, r#""#, Processor::LSR,  Operation::LSR, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BVC"#, r#""#, Processor::BVC,  Operation::BVC, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::INDY, AddressingMode::INDY, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"LSR"#, r#""#, Processor::LSR,  Operation::LSR, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"CLI"#, r#""#, Processor::CLI,  Operation::CLI, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"EOR"#, r#""#, Processor::EOR,  Operation::EOR, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"LSR"#, r#""#, Processor::LSR,  Operation::LSR, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"SRE"#, r#""#, Processor::SRE,  Operation::SRE, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"RTS"#, r#""#, Processor::RTS,  Operation::RTS, Processor::IMPL, AddressingMode::IMPL, 6),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDX, AddressingMode::INDX, n_c_jam),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPG,  AddressingMode::ZPG, n_c_nop),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"ROR"#, r#""#, Processor::ROR,  Operation::ROR, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"PLA"#, r#""#, Processor::PLA,  Operation::PLA, Processor::IMPL, AddressingMode::IMPL, 4),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"ROR"#, r#""#, Processor::ROR,  Operation::ROR, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ARR"#, r#""#, Processor::ARR,  Operation::ARR, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"JMP"#, r#""#, Processor::JMP,  Operation::JMP, Processor::IND,  AddressingMode::IND, 5),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"ROR"#, r#""#, Processor::ROR,  Operation::ROR, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BVS"#, r#""#, Processor::BVS,  Operation::BVS, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::INDY, AddressingMode::INDY, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"ROR"#, r#""#, Processor::ROR,  Operation::ROR, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"SEI"#, r#""#, Processor::SEI,  Operation::SEI, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"ADC"#, r#""#, Processor::ADC,  Operation::ADC, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"ROR"#, r#""#, Processor::ROR,  Operation::ROR, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"RRA"#, r#""#, Processor::RRA,  Operation::RRA, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMM,  AddressingMode::IMM, n_c_nop),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMM,  AddressingMode::IMM, n_c_nop),
            Instruction::new(r#"SAX"#, r#""#, Processor::SAX,  Operation::SAX, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"STY"#, r#""#, Processor::STY,  Operation::STY, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"STX"#, r#""#, Processor::STX,  Operation::STX, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"SAX"#, r#""#, Processor::SAX,  Operation::SAX, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"DEY"#, r#""#, Processor::DEY,  Operation::DEY, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMM,  AddressingMode::IMM, n_c_nop),
            Instruction::new(r#"TXA"#, r#""#, Processor::TXA,  Operation::TXA, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"ANE"#, r#""#, Processor::ANE,  Operation::ANE, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"STY"#, r#""#, Processor::STY,  Operation::STY, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"STX"#, r#""#, Processor::STX,  Operation::STX, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"SAX"#, r#""#, Processor::SAX,  Operation::SAX, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"BCC"#, r#""#, Processor::BCC,  Operation::BCC, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::INDY, AddressingMode::INDY, 6),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"SHA"#, r#""#, Processor::SHA,  Operation::SHA, Processor::INDY, AddressingMode::INDY, 6),
            Instruction::new(r#"STY"#, r#""#, Processor::STY,  Operation::STY, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"STX"#, r#""#, Processor::STX,  Operation::STX, Processor::ZPGY, AddressingMode::ZPGY, 4),
            Instruction::new(r#"SAX"#, r#""#, Processor::SAX,  Operation::SAX, Processor::ZPGY, AddressingMode::ZPGY, 4),
            Instruction::new(r#"TYA"#, r#""#, Processor::TYA,  Operation::TYA, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::ABSY, AddressingMode::ABSY, 5),
            Instruction::new(r#"TXS"#, r#""#, Processor::TXS,  Operation::TXS, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"TAS"#, r#""#, Processor::TAS,  Operation::TAS, Processor::ABSY, AddressingMode::ABSY, 5),
            Instruction::new(r#"SHY"#, r#""#, Processor::SHY,  Operation::SHY, Processor::ABSX, AddressingMode::ABSX, 5),
            Instruction::new(r#"STA"#, r#""#, Processor::STA,  Operation::STA, Processor::ABSX, AddressingMode::ABSX, 5),
            Instruction::new(r#"SHX"#, r#""#, Processor::SHX,  Operation::SHX, Processor::ABSY, AddressingMode::ABSY, 5),
            Instruction::new(r#"SHA"#, r#""#, Processor::SHA,  Operation::SHA, Processor::ABSY, AddressingMode::ABSY, 5),
            Instruction::new(r#"LDY"#, r#""#, Processor::LDY,  Operation::LDY, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"LDX"#, r#""#, Processor::LDX,  Operation::LDX, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"LDY"#, r#""#, Processor::LDY,  Operation::LDY, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"LDX"#, r#""#, Processor::LDX,  Operation::LDX, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"TAY"#, r#""#, Processor::TAY,  Operation::TAY, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"TAX"#, r#""#, Processor::TAX,  Operation::TAX, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"LXA"#, r#""#, Processor::LXA,  Operation::LXA, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"LDY"#, r#""#, Processor::LDY,  Operation::LDY, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"LDX"#, r#""#, Processor::LDX,  Operation::LDX, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"BCS"#, r#""#, Processor::BCS,  Operation::BCS, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"LDY"#, r#""#, Processor::LDY,  Operation::LDY, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"LDX"#, r#""#, Processor::LDX,  Operation::LDX, Processor::ZPGY, AddressingMode::ZPGY, 4),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::ZPGY, AddressingMode::ZPGY, 4),
            Instruction::new(r#"CLV"#, r#""#, Processor::CLV,  Operation::CLV, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"TSX"#, r#""#, Processor::TSX,  Operation::TSX, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"LAS"#, r#""#, Processor::LAS,  Operation::LAS, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"LDY"#, r#""#, Processor::LDY,  Operation::LDY, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"LDA"#, r#""#, Processor::LDA,  Operation::LDA, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"LDX"#, r#""#, Processor::LDX,  Operation::LDX, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"LAX"#, r#""#, Processor::LAX,  Operation::LAX, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"CPY"#, r#""#, Processor::CPY,  Operation::CPY, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMM,  AddressingMode::IMM, n_c_nop),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"CPY"#, r#""#, Processor::CPY,  Operation::CPY, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"DEC"#, r#""#, Processor::DEC,  Operation::DEC, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"INY"#, r#""#, Processor::INY,  Operation::INY, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"DEX"#, r#""#, Processor::DEX,  Operation::DEX, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"SBX"#, r#""#, Processor::SBX,  Operation::SBX, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"CPY"#, r#""#, Processor::CPY,  Operation::CPY, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"DEC"#, r#""#, Processor::DEC,  Operation::DEC, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BNE"#, r#""#, Processor::BNE,  Operation::BNE, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::INDY, AddressingMode::INDY, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"DEC"#, r#""#, Processor::DEC,  Operation::DEC, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"CLD"#, r#""#, Processor::CLD,  Operation::CLD, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"CMP"#, r#""#, Processor::CMP,  Operation::CMP, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"DEC"#, r#""#, Processor::DEC,  Operation::DEC, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"DCP"#, r#""#, Processor::DCP,  Operation::DCP, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"CPX"#, r#""#, Processor::CPX,  Operation::CPX, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::INDX, AddressingMode::INDX, 6),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMM,  AddressingMode::IMM, n_c_nop),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::INDX, AddressingMode::INDX, 8),
            Instruction::new(r#"CPX"#, r#""#, Processor::CPX,  Operation::CPX, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::ZPG,  AddressingMode::ZPG, 3),
            Instruction::new(r#"INC"#, r#""#, Processor::INC,  Operation::INC, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::ZPG,  AddressingMode::ZPG, 5),
            Instruction::new(r#"INX"#, r#""#, Processor::INX,  Operation::INX, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::IMM,  AddressingMode::IMM, 2),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"USBC"#, r#""#, Processor::USBC, Operation::USBC, Processor::IMM,AddressingMode::IMM, 2),
            Instruction::new(r#"CPX"#, r#""#, Processor::CPX,  Operation::CPX, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::ABS,  AddressingMode::ABS, 4),
            Instruction::new(r#"INC"#, r#""#, Processor::INC,  Operation::INC, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::ABS,  AddressingMode::ABS, 6),
            Instruction::new(r#"BEQ"#, r#""#, Processor::BEQ,  Operation::BEQ, Processor::REL,  AddressingMode::REL, 2),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::INDY, AddressingMode::INDY, 5),
            Instruction::new(r#"JAM"#, r#""#, Processor::JAM,  Operation::JAM, Processor::INDY, AddressingMode::INDY, n_c_jam),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::INDY, AddressingMode::INDY, 8),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ZPGX, AddressingMode::ZPGX, n_c_nop),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::ZPGX, AddressingMode::ZPGX, 4),
            Instruction::new(r#"INC"#, r#""#, Processor::INC,  Operation::INC, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::ZPGX, AddressingMode::ZPGX, 6),
            Instruction::new(r#"SED"#, r#""#, Processor::SED,  Operation::SED, Processor::IMPL, AddressingMode::IMPL, 2),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::ABSY, AddressingMode::ABSY, 4),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::IMPL, AddressingMode::IMPL, n_c_nop),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::ABSY, AddressingMode::ABSY, 7),
            Instruction::new(r#"NOP"#, r#""#, Processor::NOP,  Operation::NOP, Processor::ABSX, AddressingMode::ABSX, n_c_nop),
            Instruction::new(r#"SBC"#, r#""#, Processor::SBC,  Operation::SBC, Processor::ABSX, AddressingMode::ABSX, 4),
            Instruction::new(r#"INC"#, r#""#, Processor::INC,  Operation::INC, Processor::ABSX, AddressingMode::ABSX, 7),
            Instruction::new(r#"ISC"#, r#""#, Processor::ISC,  Operation::ISC, Processor::ABSX, AddressingMode::ABSX, 7),
        ]
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /**
     * Whether or not the getters for status register work properly
     */
    #[test]
    fn get_status() {
        let mut test_processor = Processor::new();
        test_processor.reset();

        // testing with values from 0 to 255
        for value in 0..=255 as u8 {

            // testing get
            test_processor.status = value;
            assert_eq!(test_processor.get_c(), (test_processor.status & (1 << CARRY_POS) == (1 << CARRY_POS)));
            assert_eq!(test_processor.get_z(), (test_processor.status & (1 << ZERO_POS) == (1 << ZERO_POS)));
            assert_eq!(test_processor.get_i(), (test_processor.status & (1 << INTERRUPT_DISABLE_POS) == (1 << INTERRUPT_DISABLE_POS)));
            assert_eq!(test_processor.get_d(), (test_processor.status & (1 << DECIMAL_POS) == (1 << DECIMAL_POS)));
            assert_eq!(test_processor.get_b(), (test_processor.status & (1 << B_FLAG_POS) == (1 << B_FLAG_POS)));
            assert_eq!(test_processor.get_u(), (test_processor.status & (1 << UNUSED_FLAG_POS) == (1 << UNUSED_FLAG_POS)));
            assert_eq!(test_processor.get_o(), (test_processor.status & (1 << OVERFLOW_POS) == (1 << OVERFLOW_POS)));
            assert_eq!(test_processor.get_n(), (test_processor.status & (1 << NEGATIVE_POS) == (1 << NEGATIVE_POS)));
        }
    }

    /**
     * Whether or not the setters for status register work properly, given getters work properly
     */
    #[test]
    fn set_status() {
        let mut test_processor = Processor::new();
        test_processor.reset();

        let boolean_values = [true, false];

        for value in boolean_values.iter() {
            // testing set
            test_processor.set_b(value.to_owned());
            assert_eq!(test_processor.get_b(), value.to_owned());

            test_processor.set_c(value.to_owned());
            assert_eq!(test_processor.get_c(), value.to_owned());

            test_processor.set_d(value.to_owned());
            assert_eq!(test_processor.get_d(), value.to_owned());

            test_processor.set_i(value.to_owned());
            assert_eq!(test_processor.get_i(), value.to_owned());

            test_processor.set_n(value.to_owned());
            assert_eq!(test_processor.get_n(), value.to_owned());

            test_processor.set_o(value.to_owned());
            assert_eq!(test_processor.get_o(), value.to_owned());

            test_processor.set_u(value.to_owned());
            assert_eq!(test_processor.get_u(), value.to_owned());

            test_processor.set_z(value.to_owned());
            assert_eq!(test_processor.get_z(), value.to_owned());
        }
        
    }


}
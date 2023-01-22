use crate::bus::Bus;
use crate::memory::Memory;

// Status bits
// representing the number of left shift required to get the bit from 0x01
const CARRY_POS: u8 = 0; // c
const ZERO_POS: u8 = 1; // z
const INTERRUPT_DISABLE_POS: u8 = 2; // i
const DECIMAL_POS: u8 = 3; // d
const B_FLAG_POS: u8 = 4; // b
const OVERFLOW_POS: u8 = 5; // o
const NEGATIVE_POS: u8 = 6; // n

// Sizes
// Setting the size to be 64 KB, in case of 6502
const KB: usize = 1024;

const RAM: usize = 16 * KB; // 16 KB RAM    (0 - 16 * 1024-1) ( 0x0000 - 0x3FFF)
const OTHER: usize = 16 * KB; // 16 KB other addressed (like 16KB, but the addresses might be other devices) (16* 1024 - 32 * 1024) (0x4000 - 0x7FFF)
const ROM: usize = 32 * KB; // 32 KB ROM (32 * 1024 - 64 * 1024) (0x8000 - 0xFFFF)

// 6502
/**
* 6502 is little endian, valid for 16 bit addresses
* i.e. $LLHH
*/
pub struct Processor {
    bus: Bus<u8>,

    // Instruction set
    instructions: Vec<Instruction>,

    // CPU core registers
    accumulator: u8,
    index_register_x: u8,
    index_register_y: u8,
    status: u8,
    stack_pointer: u8,
    program_counter: u16,

    // some variables for implementing adressing mode and opcodes
    fetched: u8,
    temp: u8,
    address_absolute: u16,
    address_relative: u8,
    opcode: u8,
    cycles: u8,
}

/*
The default value for memory field is memory with given size (bytes)
and for the instruction set, the default value if from `create_instruction_table()` function of instructions struct
Everything else defaults to zero for integers
*/
impl Default for Processor {
    fn default() -> Self {
        Self {
            bus: Bus::new(Memory::new(RAM), vec![0; OTHER], vec![0; ROM]),
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
                (self.bus.read(pointer_high) << 8) as u16 | self.bus.read(pointer + 0) as u16;
        } else {
            // behave normally
            self.address_absolute =
                ((self.bus.read(pointer + 1)) << 8) as u16 | self.bus.read(pointer + 0) as u16;
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

        self.address_absolute =
            (self.bus.read(actual_pointer + 1) << 8) as u16 | self.bus.read(actual_pointer) as u16;

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
        self.address_relative = self.bus.read(self.program_counter);
        self.program_counter += 1;

        // if (self.address_relative & 0x80) {self.address_relative |= 0xFF00}

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

/*
 * OP Codes implementation
 */
#[allow(non_snake_case)]
impl Processor {
    // add with carry
    fn ADC() -> bool {
        true
    }

    // and (with accumulator)
    fn AND() -> bool {
        true
    }

    // arithmetic shift left
    fn ASL() -> bool {
        true
    }

    // branch on carry clear
    fn BCC() -> bool {
        true
    }

    // branch on carry set
    fn BCS() -> bool {
        true
    }

    // branch on equal (zero set)
    fn BEQ() -> bool {
        true
    }

    // bit test
    fn BIT() -> bool {
        true
    }

    // branch on minus (negative set)
    fn BMI() -> bool {
        true
    }

    // branch on not equal (zero clear)
    fn BNE() -> bool {
        true
    }

    // branch on plus (negative clear)
    fn BPL() -> bool {
        true
    }

    // break / interrupt
    fn BRK() -> bool {
        true
    }

    // branch on overflow clear
    fn BVC() -> bool {
        true
    }

    // branch on overflow set
    fn BVS() -> bool {
        true
    }

    // clear carry
    fn CLC() -> bool {
        true
    }

    // clear decimal
    fn CLD() -> bool {
        true
    }

    // clear interrupt disable
    fn CLI() -> bool {
        true
    }

    // clear overflow
    fn CLV() -> bool {
        true
    }

    // compare (with accumulator)
    fn CMP() -> bool {
        true
    }

    // compare with X
    fn CPX() -> bool {
        true
    }

    // compare with Y
    fn CPY() -> bool {
        true
    }

    // decrement
    fn DEC() -> bool {
        true
    }

    // decrement X
    fn DEX() -> bool {
        true
    }

    // decrement Y
    fn DEY() -> bool {
        true
    }

    // exclusive or (with accumulator)
    fn EOR() -> bool {
        true
    }

    // increment
    fn INC() -> bool {
        true
    }

    // increment X
    fn INX() -> bool {
        true
    }

    // increment Y
    fn INY() -> bool {
        true
    }

    // jump
    fn JMP() -> bool {
        true
    }

    // jump subroutine
    fn JSR() -> bool {
        true
    }

    // load accumulator
    fn LDA() -> bool {
        true
    }

    // load X
    fn LDX() -> bool {
        true
    }

    // load Y
    fn LDY() -> bool {
        true
    }

    // logical shift right
    fn LSR() -> bool {
        true
    }

    // no operation
    fn NOP() -> bool {
        true
    }

    // or with accumulator
    fn ORA() -> bool {
        true
    }

    // push accumulator
    fn PHA() -> bool {
        true
    }

    // push processor status (SR)
    fn PHP() -> bool {
        true
    }

    // pull accumulator
    fn PLA() -> bool {
        true
    }

    // pull processor status (SR)
    fn PLP() -> bool {
        true
    }

    // rotate left
    fn ROL() -> bool {
        true
    }

    // rotate right
    fn ROR() -> bool {
        true
    }

    // return from interrupt
    fn RTI() -> bool {
        true
    }

    // return from subroutine
    fn RTS() -> bool {
        true
    }

    // subtract with carry
    fn SBC() -> bool {
        true
    }

    // set carry
    fn SEC() -> bool {
        true
    }

    // set decimal
    fn SED() -> bool {
        true
    }

    // set interrupt disable
    fn SEI() -> bool {
        true
    }

    // store accumulator
    fn STA() -> bool {
        true
    }

    // store X
    fn STX() -> bool {
        true
    }

    // store Y
    fn STY() -> bool {
        true
    }

    // transfer accumulator to X
    fn TAX() -> bool {
        true
    }

    // transfer accumulator to Y
    fn TAY() -> bool {
        true
    }

    // transfer stack pointer to X
    fn TSX() -> bool {
        true
    }

    // transfer X to accumulator
    fn TXA() -> bool {
        true
    }

    // transfer X to stack pointer
    fn TXS() -> bool {
        true
    }

    // transfer Y to accumulator
    fn TYA() -> bool {
        true
    }
}

struct Instruction {
    name: String,
    human_readable_form: String,
    operation: fn() -> bool,
    addressing_mode: fn(&mut Processor) -> bool,
    cycles: u8,
}

impl Instruction {
    fn new(
        name: &str,
        human_readable_form: &str,
        operation: fn() -> bool,
        addressing_mode: fn(&mut Processor) -> bool,
        cycles: u8,
    ) -> Self {
        Self {
            name: String::from(name),
            human_readable_form: String::from(human_readable_form),
            operation,
            addressing_mode,
            cycles,
        }
    }

    fn create_instructions_table() -> Vec<Instruction> {
        // creating instructions
        vec![
            Instruction::new(
                r#"BRK"#,
                r#"Break / Interrupt"#,
                Processor::BRK,
                Processor::IMPL,
                7,
            ),
            Instruction::new(
                r#"ORA"#,
                r#"OR with accumulator"#,
                Processor::ORA,
                Processor::INDX,
                6,
            ),
        ]
    }
}

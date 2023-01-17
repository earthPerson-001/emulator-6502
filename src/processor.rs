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
const N_KB: usize = 64;

enum AddressingMode {
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

enum Operation {
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
}

struct Instruction {
    name: String,
    human_readable_form: String,
    operation: Operation,
    addressing_mode: AddressingMode,
    cycles: u8,
}

impl Instruction {
    fn new(
        name: &str,
        human_readable_form: &str,
        operation: Operation,
        addressing_mode: AddressingMode,
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
                Operation::BRK,
                AddressingMode::IMPL,
                7,
            ),
            Instruction::new(
                r#"ORA"#,
                r#"OR with accumulator"#,
                Operation::ORA,
                AddressingMode::INDX,
                6,
            ),
        ]
    }
}

// 6502
/// * 6502 is little endian, valid for 16 bit addresses
/// * i.e. $LLHH
pub struct Processor {
    memory: Memory,
    accumulator: u8,
    index_register_x: u8,
    index_register_y: u8,
    status: u8,
    stack_pointer: u8,
    program_counter: u16,
    instructions: Vec<Instruction>,
}

// Constructor like implementation
impl Processor {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(N_KB * KB),
            accumulator: 0x00,
            index_register_x: 0x00,
            index_register_y: 0x00,
            status: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
            instructions: Instruction::create_instructions_table(),
        }
    }
}

/**
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

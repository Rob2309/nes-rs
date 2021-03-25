use crate::{cpu::{AddressingMode, Cpu}, memory::Memory};

/// A Function emulating a single CPU instruction
/// - `addr_mode`: the concrete [`AddressingMode`] the instruction is using (allows for multiple instruction encodings using the same functions)
/// - `memory`: a [`Memory`] object that can be used to access CPU and PPU memory
pub(crate) type CpuOpFunc = fn (&mut Cpu, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8;

/// Describes a single CPU instruction and its encoding
#[derive(Clone, Copy)]
pub(crate) struct CpuOp {
    /// Mnemonic of the instruction (used for debugging)
    pub name: &'static str,
    /// 8-Bit opcode of the instruction, as used by the CPU
    pub opcode: u8,
    /// [`AddressingMode`] of the instruction (describes which operands it takes)
    pub addr_mode: AddressingMode,
    /// The function that emulates this instruction, see [`CpuOpFunc`]
    pub func: CpuOpFunc
}

/// Collection of all *official* CPU instructions
pub(crate) const CPU_OPS: [CpuOp; 151] = [
    CpuOp { name: "ADC", opcode: 0x69, addr_mode: AddressingMode::Immediate, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x65, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x75, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x6D, addr_mode: AddressingMode::Absolute, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x7D, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x79, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x61, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_adc },
    CpuOp { name: "ADC", opcode: 0x71, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_adc },

    CpuOp { name: "AND", opcode: 0x29, addr_mode: AddressingMode::Immediate, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x25, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x35, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x2D, addr_mode: AddressingMode::Absolute, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x3D, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x39, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x21, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_and },
    CpuOp { name: "AND", opcode: 0x31, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_and },

    CpuOp { name: "ASL", opcode: 0x0A, addr_mode: AddressingMode::Implicit, func: Cpu::op_asl_a },
    CpuOp { name: "ASL", opcode: 0x06, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_asl_m },
    CpuOp { name: "ASL", opcode: 0x16, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_asl_m },
    CpuOp { name: "ASL", opcode: 0x0E, addr_mode: AddressingMode::Absolute, func: Cpu::op_asl_m },
    CpuOp { name: "ASL", opcode: 0x1E, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_asl_m },

    CpuOp { name: "BCC", opcode: 0x90, addr_mode: AddressingMode::Relative, func: Cpu::op_bcc },
    CpuOp { name: "BCS", opcode: 0xB0, addr_mode: AddressingMode::Relative, func: Cpu::op_bcs },
    CpuOp { name: "BEQ", opcode: 0xF0, addr_mode: AddressingMode::Relative, func: Cpu::op_beq },

    CpuOp { name: "BIT", opcode: 0x24, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_bit },
    CpuOp { name: "BIT", opcode: 0x2C, addr_mode: AddressingMode::Absolute, func: Cpu::op_bit },

    CpuOp { name: "BMI", opcode: 0x30, addr_mode: AddressingMode::Relative, func: Cpu::op_bmi },
    CpuOp { name: "BNE", opcode: 0xD0, addr_mode: AddressingMode::Relative, func: Cpu::op_bne },
    CpuOp { name: "BPL", opcode: 0x10, addr_mode: AddressingMode::Relative, func: Cpu::op_bpl },

    CpuOp { name: "BRK", opcode: 0x00, addr_mode: AddressingMode::Implicit, func: Cpu::op_brk },

    CpuOp { name: "BVC", opcode: 0x50, addr_mode: AddressingMode::Relative, func: Cpu::op_bvc },
    CpuOp { name: "BVS", opcode: 0x70, addr_mode: AddressingMode::Relative, func: Cpu::op_bvs },

    CpuOp { name: "CLC", opcode: 0x18, addr_mode: AddressingMode::Implicit, func: Cpu::op_clc },
    CpuOp { name: "CLD", opcode: 0xD8, addr_mode: AddressingMode::Implicit, func: Cpu::op_cld },
    CpuOp { name: "CLI", opcode: 0x58, addr_mode: AddressingMode::Implicit, func: Cpu::op_cli },
    CpuOp { name: "CLV", opcode: 0xB8, addr_mode: AddressingMode::Implicit, func: Cpu::op_clv },

    CpuOp { name: "CMP", opcode: 0xC9, addr_mode: AddressingMode::Immediate, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xC5, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xD5, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xCD, addr_mode: AddressingMode::Absolute, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xDD, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xD9, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xC1, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_cmp },
    CpuOp { name: "CMP", opcode: 0xD1, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_cmp },

    CpuOp { name: "CPX", opcode: 0xE0, addr_mode: AddressingMode::Immediate, func: Cpu::op_cpx },
    CpuOp { name: "CPX", opcode: 0xE4, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_cpx },
    CpuOp { name: "CPX", opcode: 0xEC, addr_mode: AddressingMode::Absolute, func: Cpu::op_cpx },

    CpuOp { name: "CPY", opcode: 0xC0, addr_mode: AddressingMode::Immediate, func: Cpu::op_cpy },
    CpuOp { name: "CPY", opcode: 0xC4, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_cpy },
    CpuOp { name: "CPY", opcode: 0xCC, addr_mode: AddressingMode::Absolute, func: Cpu::op_cpy },

    CpuOp { name: "DEC", opcode: 0xC6, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_dec },
    CpuOp { name: "DEC", opcode: 0xD6, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_dec },
    CpuOp { name: "DEC", opcode: 0xCE, addr_mode: AddressingMode::Absolute, func: Cpu::op_dec },
    CpuOp { name: "DEC", opcode: 0xDE, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_dec },

    CpuOp { name: "DEX", opcode: 0xCA, addr_mode: AddressingMode::Implicit, func: Cpu::op_dex },

    CpuOp { name: "DEY", opcode: 0x88, addr_mode: AddressingMode::Implicit, func: Cpu::op_dey },

    CpuOp { name: "EOR", opcode: 0x49, addr_mode: AddressingMode::Immediate, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x45, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x55, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x4D, addr_mode: AddressingMode::Absolute, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x5D, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x59, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x41, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_eor },
    CpuOp { name: "EOR", opcode: 0x51, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_eor },

    CpuOp { name: "INC", opcode: 0xE6, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_inc },
    CpuOp { name: "INC", opcode: 0xF6, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_inc },
    CpuOp { name: "INC", opcode: 0xEE, addr_mode: AddressingMode::Absolute, func: Cpu::op_inc },
    CpuOp { name: "INC", opcode: 0xFE, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_inc },

    CpuOp { name: "INX", opcode: 0xE8, addr_mode: AddressingMode::Implicit, func: Cpu::op_inx },

    CpuOp { name: "INY", opcode: 0xC8, addr_mode: AddressingMode::Implicit, func: Cpu::op_iny },

    CpuOp { name: "JMP", opcode: 0x4C, addr_mode: AddressingMode::Absolute, func: Cpu::op_jmp },
    CpuOp { name: "JMP", opcode: 0x6C, addr_mode: AddressingMode::Indirect, func: Cpu::op_jmp },

    CpuOp { name: "JSR", opcode: 0x20, addr_mode: AddressingMode::Absolute, func: Cpu::op_jsr },

    CpuOp { name: "LDA", opcode: 0xA9, addr_mode: AddressingMode::Immediate, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xA5, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xB5, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xAD, addr_mode: AddressingMode::Absolute, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xBD, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xB9, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xA1, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_lda },
    CpuOp { name: "LDA", opcode: 0xB1, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_lda },

    CpuOp { name: "LDX", opcode: 0xA2, addr_mode: AddressingMode::Immediate, func: Cpu::op_ldx },
    CpuOp { name: "LDX", opcode: 0xA6, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_ldx },
    CpuOp { name: "LDX", opcode: 0xB6, addr_mode: AddressingMode::ZeroPageY, func: Cpu::op_ldx },
    CpuOp { name: "LDX", opcode: 0xAE, addr_mode: AddressingMode::Absolute, func: Cpu::op_ldx },
    CpuOp { name: "LDX", opcode: 0xBE, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_ldx },

    CpuOp { name: "LDY", opcode: 0xA0, addr_mode: AddressingMode::Immediate, func: Cpu::op_ldy },
    CpuOp { name: "LDY", opcode: 0xA4, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_ldy },
    CpuOp { name: "LDY", opcode: 0xB4, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_ldy },
    CpuOp { name: "LDY", opcode: 0xAC, addr_mode: AddressingMode::Absolute, func: Cpu::op_ldy },
    CpuOp { name: "LDY", opcode: 0xBC, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_ldy },

    CpuOp { name: "LSR", opcode: 0x4A, addr_mode: AddressingMode::Implicit, func: Cpu::op_lsr_a },
    CpuOp { name: "LSR", opcode: 0x46, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_lsr_m },
    CpuOp { name: "LSR", opcode: 0x56, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_lsr_m },
    CpuOp { name: "LSR", opcode: 0x4E, addr_mode: AddressingMode::Absolute, func: Cpu::op_lsr_m },
    CpuOp { name: "LSR", opcode: 0x5E, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_lsr_m },

    CpuOp { name: "NOP", opcode: 0xEA, addr_mode: AddressingMode::Implicit, func: Cpu::op_nop },

    CpuOp { name: "ORA", opcode: 0x09, addr_mode: AddressingMode::Immediate, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x05, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x15, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x0D, addr_mode: AddressingMode::Absolute, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x1D, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x19, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x01, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_ora },
    CpuOp { name: "ORA", opcode: 0x11, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_ora },

    CpuOp { name: "PHA", opcode: 0x48, addr_mode: AddressingMode::Implicit, func: Cpu::op_pha },
    CpuOp { name: "PHP", opcode: 0x08, addr_mode: AddressingMode::Implicit, func: Cpu::op_php },
    CpuOp { name: "PLA", opcode: 0x68, addr_mode: AddressingMode::Implicit, func: Cpu::op_pla },
    CpuOp { name: "PLP", opcode: 0x28, addr_mode: AddressingMode::Implicit, func: Cpu::op_plp },

    CpuOp { name: "ROL", opcode: 0x2A, addr_mode: AddressingMode::Implicit, func: Cpu::op_rol_a },
    CpuOp { name: "ROL", opcode: 0x26, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_rol_m },
    CpuOp { name: "ROL", opcode: 0x36, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_rol_m },
    CpuOp { name: "ROL", opcode: 0x2E, addr_mode: AddressingMode::Absolute, func: Cpu::op_rol_m },
    CpuOp { name: "ROL", opcode: 0x3E, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_rol_m },

    CpuOp { name: "ROR", opcode: 0x6A, addr_mode: AddressingMode::Implicit, func: Cpu::op_ror_a },
    CpuOp { name: "ROR", opcode: 0x66, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_ror_m },
    CpuOp { name: "ROR", opcode: 0x76, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_ror_m },
    CpuOp { name: "ROR", opcode: 0x6E, addr_mode: AddressingMode::Absolute, func: Cpu::op_ror_m },
    CpuOp { name: "ROR", opcode: 0x7E, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_ror_m },

    CpuOp { name: "RTI", opcode: 0x40, addr_mode: AddressingMode::Implicit, func: Cpu::op_rti },

    CpuOp { name: "RTS", opcode: 0x60, addr_mode: AddressingMode::Implicit, func: Cpu::op_rts },

    CpuOp { name: "SBC", opcode: 0xE9, addr_mode: AddressingMode::Immediate, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xE5, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xF5, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xED, addr_mode: AddressingMode::Absolute, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xFD, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xF9, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xE1, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_sbc },
    CpuOp { name: "SBC", opcode: 0xF1, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_sbc },

    CpuOp { name: "SEC", opcode: 0x38, addr_mode: AddressingMode::Implicit, func: Cpu::op_sec },
    CpuOp { name: "SED", opcode: 0xF8, addr_mode: AddressingMode::Implicit, func: Cpu::op_sed },
    CpuOp { name: "SEI", opcode: 0x78, addr_mode: AddressingMode::Implicit, func: Cpu::op_sei },

    CpuOp { name: "STA", opcode: 0x85, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x95, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x8D, addr_mode: AddressingMode::Absolute, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x9D, addr_mode: AddressingMode::AbsoluteX, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x99, addr_mode: AddressingMode::AbsoluteY, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x81, addr_mode: AddressingMode::IndexedIndirect, func: Cpu::op_sta },
    CpuOp { name: "STA", opcode: 0x91, addr_mode: AddressingMode::IndirectIndexed, func: Cpu::op_sta },

    CpuOp { name: "STX", opcode: 0x86, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_stx },
    CpuOp { name: "STX", opcode: 0x96, addr_mode: AddressingMode::ZeroPageY, func: Cpu::op_stx },
    CpuOp { name: "STX", opcode: 0x8E, addr_mode: AddressingMode::Absolute, func: Cpu::op_stx },

    CpuOp { name: "STY", opcode: 0x84, addr_mode: AddressingMode::ZeroPage, func: Cpu::op_sty },
    CpuOp { name: "STY", opcode: 0x94, addr_mode: AddressingMode::ZeroPageX, func: Cpu::op_sty },
    CpuOp { name: "STY", opcode: 0x8C, addr_mode: AddressingMode::Absolute, func: Cpu::op_sty },

    CpuOp { name: "TAX", opcode: 0xAA, addr_mode: AddressingMode::Implicit, func: Cpu::op_tax },
    CpuOp { name: "TAY", opcode: 0xA8, addr_mode: AddressingMode::Implicit, func: Cpu::op_tay },
    CpuOp { name: "TSX", opcode: 0xBA, addr_mode: AddressingMode::Implicit, func: Cpu::op_tsx },
    CpuOp { name: "TXA", opcode: 0x8A, addr_mode: AddressingMode::Implicit, func: Cpu::op_txa },
    CpuOp { name: "TXS", opcode: 0x9A, addr_mode: AddressingMode::Implicit, func: Cpu::op_txs },
    CpuOp { name: "TYA", opcode: 0x98, addr_mode: AddressingMode::Implicit, func: Cpu::op_tya },
];

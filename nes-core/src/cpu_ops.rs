use crate::{cpu::{AddressingMode, Cpu}, memory::Memory};

pub type CpuOpFunc = fn (&mut Cpu, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8;

pub struct CpuOp {
    opcode: u8,
    addr_mode: AddressingMode,
    cycles: u8,
    func: CpuOpFunc
}

pub const CPU_OPS: [CpuOp; 36] = [
    CpuOp { opcode: 0x69, addr_mode: AddressingMode::Immediate, cycles: 2, func: Cpu::op_adc },
    CpuOp { opcode: 0x65, addr_mode: AddressingMode::ZeroPage, cycles: 3, func: Cpu::op_adc },
    CpuOp { opcode: 0x75, addr_mode: AddressingMode::ZeroPageX, cycles: 4, func: Cpu::op_adc },
    CpuOp { opcode: 0x6D, addr_mode: AddressingMode::Absolute, cycles: 4, func: Cpu::op_adc },
    CpuOp { opcode: 0x7D, addr_mode: AddressingMode::AbsoluteX, cycles: 4, func: Cpu::op_adc },
    CpuOp { opcode: 0x79, addr_mode: AddressingMode::AbsoluteY, cycles: 4, func: Cpu::op_adc },
    CpuOp { opcode: 0x61, addr_mode: AddressingMode::IndexedIndirect, cycles: 6, func: Cpu::op_adc },
    CpuOp { opcode: 0x71, addr_mode: AddressingMode::IndirectIndexed, cycles: 5, func: Cpu::op_adc },

    CpuOp { opcode: 0x29, addr_mode: AddressingMode::Immediate, cycles: 2, func: Cpu::op_and },
    CpuOp { opcode: 0x25, addr_mode: AddressingMode::ZeroPage, cycles: 3, func: Cpu::op_and },
    CpuOp { opcode: 0x35, addr_mode: AddressingMode::ZeroPageX, cycles: 4, func: Cpu::op_and },
    CpuOp { opcode: 0x2D, addr_mode: AddressingMode::Absolute, cycles: 4, func: Cpu::op_and },
    CpuOp { opcode: 0x3D, addr_mode: AddressingMode::AbsoluteX, cycles: 4, func: Cpu::op_and },
    CpuOp { opcode: 0x39, addr_mode: AddressingMode::AbsoluteY, cycles: 4, func: Cpu::op_and },
    CpuOp { opcode: 0x21, addr_mode: AddressingMode::IndexedIndirect, cycles: 6, func: Cpu::op_and },
    CpuOp { opcode: 0x31, addr_mode: AddressingMode::IndirectIndexed, cycles: 5 , func: Cpu::op_and },

    CpuOp { opcode: 0x0A, addr_mode: AddressingMode::Implicit, cycles: 2, func: Cpu::op_asl_a },
    CpuOp { opcode: 0x06, addr_mode: AddressingMode::ZeroPage, cycles: 5, func: Cpu::op_asl_m },
    CpuOp { opcode: 0x16, addr_mode: AddressingMode::ZeroPageX, cycles: 6, func: Cpu::op_asl_m },
    CpuOp { opcode: 0x0E, addr_mode: AddressingMode::Absolute, cycles: 6, func: Cpu::op_asl_m },
    CpuOp { opcode: 0x1E, addr_mode: AddressingMode::AbsoluteX, cycles: 7, func: Cpu::op_asl_m },

    CpuOp { opcode: 0x90, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bcc },
    CpuOp { opcode: 0xB0, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bcs },
    CpuOp { opcode: 0xF0, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_beq },

    CpuOp { opcode: 0x24, addr_mode: AddressingMode::ZeroPage, cycles: 3, func: Cpu::op_bit },
    CpuOp { opcode: 0x2C, addr_mode: AddressingMode::Absolute, cycles: 4, func: Cpu::op_bit },

    CpuOp { opcode: 0x30, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bmi },
    CpuOp { opcode: 0xD0, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bne },
    CpuOp { opcode: 0x10, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bpl },

    CpuOp { opcode: 0x00, addr_mode: AddressingMode::Implicit, cycles: 7, func: Cpu::op_brk },

    CpuOp { opcode: 0x50, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bvc },
    CpuOp { opcode: 0x70, addr_mode: AddressingMode::Relative, cycles: 2, func: Cpu::op_bvs },

    CpuOp { opcode: 0x18, addr_mode: AddressingMode::Implicit, cycles: 2, func: Cpu::op_clc },
    CpuOp { opcode: 0xD8, addr_mode: AddressingMode::Implicit, cycles: 2, func: Cpu::op_cld },
    CpuOp { opcode: 0x58, addr_mode: AddressingMode::Implicit, cycles: 2, func: Cpu::op_cli },
    CpuOp { opcode: 0xB8, addr_mode: AddressingMode::Implicit, cycles: 2, func: Cpu::op_clv },
];

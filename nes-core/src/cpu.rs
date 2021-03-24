use crate::{cpu_ops::{self, CPU_OPS, CpuOp}, memory::Memory};

pub const CPU_CLOCK_DIV: i32 = 12;

pub struct Cpu {
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_pc: u16,
    reg_s: u8,
    reg_p: u8,

    cycle_counter: u64,
    remaining_cycles: u8,

    opmap: [CpuOp; 0x100],
}

impl Cpu {
    pub fn new() -> Self {
        let mut opmap = [CpuOp{ name: "???", opcode: 0x00, addr_mode: AddressingMode::Implicit, cycles: 1, func: Self::op_invalid}; 0x100];

        for op in &CPU_OPS {
            opmap[op.opcode as usize] = *op;
        }
        
        Self {
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_pc: 0,
            reg_s: 0,
            reg_p: 0,

            cycle_counter: 0,
            remaining_cycles: 1,

            opmap
        }
    }

    /// Resets the CPU to the following state
    /// - P: InterruptDisable
    /// - A, X, Y: 0
    /// - S: 0xFD
    /// - PC: loaded from reset vector (0xFFFC)
    ///
    /// The reset will take 7 clock cycles
    pub fn reset(&mut self, memory: &mut dyn Memory) {
        self.reg_p = Flags::InterruptDisable as u8;
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_s = 0xFD;
        
        self.reg_pc = memory.load16(0xFFFC);

        self.cycle_counter = 0;
        self.remaining_cycles = 7;
    }

    /// Performs a single CPU Cycle
    /// # Instruction emulation
    /// Each instruction is executed instantaniously, meaning that every instruction is
    /// simulated in a single clock cycle, while the remaining cycles are spent waiting.
    /// This means that the emulation currently is not cycle accurate.
    pub fn cycle(&mut self, memory: &mut dyn Memory) {
        self.remaining_cycles -= 1;
        self.cycle_counter += 1;

        if self.remaining_cycles == 0 {
            let opcode = memory.load8(self.reg_pc);
            let op = self.opmap[opcode as usize];

            println!("{:0>4X}  {}  A:{:0>2X} X:{:0>2X} Y:{:0>2X} P:{:0>2X} SP:{:0>2X}  CYC:{}", self.reg_pc, op.name, self.reg_a, self.reg_x, self.reg_y, self.reg_p | 0x20, self.reg_s, self.cycle_counter);

            self.reg_pc += 1;
            
            // extra_cycles will be non-zero if the given instruction caused an 'oops' cycle,
            // or if a branch instruction took the branch (+ another cycle if the branch crosses a page)
            let extra_cycles = (op.func)(self, op.addr_mode, memory);
            self.remaining_cycles = op.cycles + extra_cycles;
        }
    }

    pub(crate) fn op_invalid(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        0
    }

    /// Sets the given flag to `value`.
    /// See [`Flags`]
    fn set_flag(&mut self, flag: Flags, value: bool) {
        if value {
            self.reg_p |= flag as u8;
        } else {
            self.reg_p &= !(flag as u8);
        }
    }
    /// Gets the value of the given flag.
    /// See [`Flags`]
    fn get_flag(&self, flag: Flags) -> bool {
        (self.reg_p & flag as u8) != 0
    }

    /// Returns the operand address for [`AddressingModes`](AddressingMode) that
    /// load an operand from memory
    /// # Returns
    /// (addr, extra_cycle)
    /// - `addr`: the resolved address of the instruction operand
    /// - `extra_cycle`: whether the addressing mode caused an extra cycle on a reading instruction
    /// # Panics
    /// This function panics when `addr_mode` is [`AddressingMode::Implicit`]
    fn get_operand_addr(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> (u16, bool) {
        match addr_mode {
            AddressingMode::ZeroPage => {
                // load immediate 1 byte address
                let arg = memory.load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                (arg as u16, false)
            }
            AddressingMode::ZeroPageX => {
                // load immediate 1 byte address
                let mut arg = memory.load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                // add x
                arg = arg.wrapping_add(self.reg_x);
                (arg as u16, false)
            }
            AddressingMode::ZeroPageY => {
                // load immediate 1 byte address
                let mut arg = memory.load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                // add y
                arg = arg.wrapping_add(self.reg_y);
                (arg as u16, false)
            }
            AddressingMode::Absolute => {
                // load immediate 2 byte address
                let arg = memory.load16(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(2);
                (arg, false)
            }
            AddressingMode::AbsoluteX => {
                // load immediate 2 byte address
                let arg = memory.load16(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(2);
                // add x register
                let final_addr = arg.wrapping_add(self.reg_x as u16);
                // if adding x changes the memory page, add an extra cycle
                let extra_cycle = (final_addr & 0xFF00) != (arg & 0xFF00);
                (final_addr, extra_cycle)
            }
            AddressingMode::AbsoluteY => {
                // load immediate 2 byte address
                let arg = memory.load16(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(2);
                // add y register
                let final_addr = arg.wrapping_add(self.reg_y as u16);
                // if adding y changes the memory page, add an extra cycle
                let extra_cycle = (final_addr & 0xFF00) != (arg & 0xFF00);
                (final_addr, extra_cycle)
            }
            AddressingMode::Immediate | AddressingMode::Relative => {
                // operand directly follows current instruction
                let addr = self.reg_pc;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                (addr, false)
            }
            AddressingMode::Indirect => {
                // load 2 byte indirect address
                let ind_addr = memory.load16(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(2);

                // Cpu bug: if ind_addr is 0x##ff, the second byte is fetched from 0x##00 instead of one page up
                // e.g. if ind_addr is 0x34FF, the bytes are loaded from 0x34FF and 0x3400 instead of 0x3500
                if (ind_addr & 0xFF) == 0xFF {
                    let addr_low = memory.load8(ind_addr);
                    let addr_high = memory.load8(ind_addr & 0xFF00);
                    let addr = ((addr_high as u16) << 8) | (addr_low as u16);
                    (addr, false)
                } else {
                    let addr = memory.load16(ind_addr);
                    (addr, false)
                }
            }
            AddressingMode::IndexedIndirect => {
                // load immediate 1 byte address
                let mut ind_addr = memory.load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);

                // add x register
                ind_addr = ind_addr.wrapping_add(self.reg_x);

                // careful: loading 2 byte address still wraps inside zero page
                let addr_low = memory.load8(ind_addr as u16);
                let addr_high = memory.load8(ind_addr.wrapping_add(1) as u16);
                let addr = ((addr_high as u16) << 8) | (addr_low as u16);
                (addr, false)
            }
            AddressingMode::IndirectIndexed => {
                // load immediate 1 byte address
                let ind_addr = memory.load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);

                // load 2 byte address
                // careful: loading wraps around in zero page
                // e.g. ($ff),Y loads absolute address from 0xff and 0x00 instead of 0x100
                let addr_low = memory.load8(ind_addr as u16);
                let addr_high = memory.load8((ind_addr.wrapping_add(1)) as u16);
                let addr = ((addr_high as u16) << 8) | (addr_low as u16);
                
                // add y register
                let final_addr = addr.wrapping_add(self.reg_y as u16);

                // if adding y causes a page change, add one cycle
                let extra_cycle = (final_addr & 0xFF00) != (addr & 0xFF00);

                (final_addr, extra_cycle)
            }
            _ => {
                panic!("Unexpected Addressing mode passed to get_operand_addr: {:?}", addr_mode);
            }
        }
    }

    pub(crate) fn op_adc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let carry_in: u16 = if self.get_flag(Flags::Carry) { 1 } else { 0 };

        let res = (op as u16).wrapping_add(self.reg_a as u16).wrapping_add(carry_in);

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        let overflow = (!(self.reg_a ^ op)) & (self.reg_a ^ (res & 0xFF) as u8) & 0x80;
        self.set_flag(Flags::Overflow, overflow != 0);

        self.reg_a = (res & 0xFF) as u8;

        extra_cycle as u8
    }

    pub(crate) fn op_and(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = self.reg_a & op;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = res;

        extra_cycle as u8
    }

    pub(crate) fn op_asl_a(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        let res = (self.reg_a as u16) << 1;

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = (res & 0xFF) as u8;
        0
    }

    pub(crate) fn op_asl_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = (op as u16) << 1;

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, (res & 0xFF) as u8);
        0
    }

    /// Performs a relative branch with `op` as signed 8-Bit Offset
    /// # Cycles
    /// - A branch instruction that does not branch takes 2 Cycles
    /// - If a branch is taken, add one cycle
    /// - If the branch crosses a page (e.g. 0x01xx -> 0x02xx), add another cycle
    fn relative_branch(&mut self, op: u8) -> u8 {
        let mut offs = op as u16;
        // perform sign extension
        if (offs & 0x80) != 0 {
            offs |= 0xFF00;
        }

        let new_pc = self.reg_pc.wrapping_add(offs);

        let old_pc = self.reg_pc;
        self.reg_pc = new_pc;

        // check if branch is across pages
        if (old_pc & 0xFF00) != (new_pc & 0xFF00) {
            2
        } else {
            1
        }
    }

    pub(crate) fn op_bcc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if !self.get_flag(Flags::Carry) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_bcs(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if self.get_flag(Flags::Carry) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_beq(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if self.get_flag(Flags::Zero) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_bit(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = self.reg_a & op;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Overflow, (op & 0x40) != 0);
        self.set_flag(Flags::Negative, (op & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_bmi(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if self.get_flag(Flags::Negative) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_bne(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if !self.get_flag(Flags::Zero) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_bpl(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if !self.get_flag(Flags::Negative) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_brk(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let ret_addr_low = (self.reg_pc & 0xFF) as u8;
        let ret_addr_high = (self.reg_pc.wrapping_shr(8)) as u8;
        let p = self.reg_p | 0x30;

        self.push(ret_addr_high, memory);
        self.push(ret_addr_low, memory);
        self.push(p, memory);

        self.set_flag(Flags::InterruptDisable, true);

        let vect = memory.load16(0xFFFE);

        self.reg_pc = vect;

        0
    }

    pub(crate) fn op_bvc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if !self.get_flag(Flags::Overflow) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_bvs(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        if self.get_flag(Flags::Overflow) {
            self.relative_branch(op)
        } else {
            0
        }
    }

    pub(crate) fn op_clc(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::Carry, false);
        0
    }

    pub(crate) fn op_cld(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::Decimal, false);
        0
    }

    pub(crate) fn op_cli(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::InterruptDisable, false);
        0
    }

    pub(crate) fn op_clv(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::Overflow, false);
        0
    }

    pub(crate) fn op_cmp(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.set_flag(Flags::Carry, self.reg_a >= op);
        self.set_flag(Flags::Zero, self.reg_a == op);

        let tmp = (self.reg_a as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_cpx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.set_flag(Flags::Carry, self.reg_x >= op);
        self.set_flag(Flags::Zero, self.reg_x == op);

        let tmp = (self.reg_x as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_cpy(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.set_flag(Flags::Carry, self.reg_y >= op);
        self.set_flag(Flags::Zero, self.reg_y == op);

        let tmp = (self.reg_y as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_dec(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = op.wrapping_sub(1);

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, res);

        0
    }

    pub(crate) fn op_dex(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_x = self.reg_x.wrapping_sub(1);

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_dey(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_y = self.reg_y.wrapping_sub(1);

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_eor(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.reg_a ^= op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_inc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = op.wrapping_add(1);

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, res);

        0
    }

    pub(crate) fn op_inx(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_x = self.reg_x.wrapping_add(1);

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_iny(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_y = self.reg_y.wrapping_add(1);

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_jmp(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);

        self.reg_pc = op_addr;

        0
    }

    pub(crate) fn op_jsr(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (jmp_addr, _) = self.get_operand_addr(addr_mode, memory);

        let ret_addr = self.reg_pc.wrapping_sub(1);
        let ret_high = ret_addr.wrapping_shr(8) as u8;
        let ret_low = (ret_addr & 0xFF) as u8;
        self.push(ret_high, memory);
        self.push(ret_low, memory);

        self.reg_pc = jmp_addr;

        0
    }

    pub(crate) fn op_lda(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.reg_a = op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_ldx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.reg_x = op;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_ldy(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.reg_y = op;

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        extra_cycle as u8
    }

    pub(crate) fn op_lsr_a(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        let res = self.reg_a.wrapping_shr(1);

        self.set_flag(Flags::Carry, (self.reg_a & 0x01) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = res;
        0
    }

    pub(crate) fn op_lsr_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let res = op.wrapping_shr(1);

        self.set_flag(Flags::Carry, (op & 0x01) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, res);
        0
    }

    pub(crate) fn op_nop(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        0
    }

    pub(crate) fn op_ora(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        self.reg_a |= op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        extra_cycle as u8
    }

    /// Pushes a byte onto the stack.
    /// 
    /// The value is pushed by
    /// 1. writing `val` to `0x0100 + reg_s`
    /// 2. decrementing `reg_s`
    /// 
    /// # Overflow
    /// The CPU does not do anything special when `reg_s` overflows,
    /// meaning the stack will loop around
    fn push(&mut self, val: u8, memory: &mut dyn Memory) {
        let addr = 0x0100 | (self.reg_s as u16);
        memory.store8(addr, val);
        self.reg_s = self.reg_s.wrapping_sub(1);
    }

    /// Pulls a byte from the stack and returns it
    /// 
    /// The value is pulled by
    /// 1. incrementing `reg_s`
    /// 2. reading from `0x0100 + reg_s`
    /// 
    /// # Returns
    /// The byte pulled from the stack
    /// 
    /// # Overflow
    /// The CPU does not do anything special when `reg_s` underflows,
    /// meaning the stack will loop around
    fn pull(&mut self, memory: &mut dyn Memory) -> u8 {
        self.reg_s = self.reg_s.wrapping_add(1);
        let addr = 0x0100 | (self.reg_s as u16);
        memory.load8(addr)
    }

    pub(crate) fn op_pha(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.push(self.reg_a, memory);
        0
    }

    pub(crate) fn op_php(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let val = self.reg_p | 0x30;
        self.push(val, memory);
        0
    }

    pub(crate) fn op_pla(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let val = self.pull(memory);
        self.reg_a = val;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_plp(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let val = self.pull(memory);
        self.reg_p = val & 0xCF;

        0
    }

    pub(crate) fn op_rol_a(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        let mut res = (self.reg_a as u16) << 1;
        if self.get_flag(Flags::Carry) {
            res |= 0x01;
        }

        self.set_flag(Flags::Carry, (res & 0x100) != 0);

        self.reg_a = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_rol_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let mut res = (op as u16) << 1;
        if self.get_flag(Flags::Carry) {
            res |= 0x01;
        }

        self.set_flag(Flags::Carry, (res & 0x100) != 0);

        let res = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, res);
        0
    }

    pub(crate) fn op_ror_a(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        let mut res = self.reg_a.wrapping_shr(1);
        if self.get_flag(Flags::Carry) {
            res |= 0x80;
        }

        self.set_flag(Flags::Carry, (self.reg_a & 0x01) != 0);

        self.reg_a = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_ror_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        let op = memory.load8(op_addr);

        let mut res = op.wrapping_shr(1);
        if self.get_flag(Flags::Carry) {
            res |= 0x80;
        }

        self.set_flag(Flags::Carry, (op & 0x01) != 0);

        let res = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.store8(op_addr, res);
        0
    }

    pub(crate) fn op_rti(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let p = self.pull(memory);
        let ret_addr_low = self.pull(memory);
        let ret_addr_high = self.pull(memory);

        let ret_addr = ((ret_addr_high as u16) << 8) | (ret_addr_low as u16);

        self.reg_p = p & 0xCF;
        self.reg_pc = ret_addr;

        0
    }

    pub(crate) fn op_rts(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let ret_addr_low = self.pull(memory);
        let ret_addr_high = self.pull(memory);

        let ret_addr = ((ret_addr_high as u16) << 8) | (ret_addr_low as u16);

        self.reg_pc = ret_addr.wrapping_add(1);

        0
    }

    pub(crate) fn op_sbc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, extra_cycle) = self.get_operand_addr(addr_mode, memory);
        let op = !memory.load8(op_addr);

        let carry_in: u16 = self.get_flag(Flags::Carry) as u16;

        let res = (op as u16).wrapping_add(self.reg_a as u16).wrapping_add(carry_in);

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        let overflow = (!(self.reg_a ^ op)) & (self.reg_a ^ (res & 0xFF) as u8) & 0x80;
        self.set_flag(Flags::Overflow, overflow != 0);

        self.reg_a = (res & 0xFF) as u8;

        extra_cycle as u8
    }

    pub(crate) fn op_sec(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::Carry, true);
        0
    }

    pub(crate) fn op_sed(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::Decimal, true);
        0
    }

    pub(crate) fn op_sei(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.set_flag(Flags::InterruptDisable, true);
        0
    }

    pub(crate) fn op_sta(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        
        memory.store8(op_addr, self.reg_a);

        0
    }

    pub(crate) fn op_stx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        
        memory.store8(op_addr, self.reg_x);

        0
    }

    pub(crate) fn op_sty(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let (op_addr, _) = self.get_operand_addr(addr_mode, memory);
        
        memory.store8(op_addr, self.reg_y);

        0
    }

    pub(crate) fn op_tax(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_x = self.reg_a;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_tay(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_y = self.reg_a;

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_tsx(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_x = self.reg_s;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_txa(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_a = self.reg_x;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_txs(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_s = self.reg_x;

        0
    }

    pub(crate) fn op_tya(&mut self, _: AddressingMode, _: &mut dyn Memory) -> u8 {
        self.reg_a = self.reg_y;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

}

/// Addressing Modes for Cpu Instructions
#[derive(Debug, Clone, Copy)]
pub(crate) enum AddressingMode {
    /// No explicit operand (e.g. INX)
    Implicit,
    /// Single byte address (e.g. ADC $7F)
    ZeroPage,
    /// Single byte address + x register (e.g. ADC $7F,X),
    /// wraps around to stay in zero page
    ZeroPageX,
    /// Single byte address + y register (e.g. ADC $7F,Y),
    /// wraps around to stay in zero page
    ZeroPageY,
    /// Two byte address (e.g. ADC $5f70)
    Absolute,
    /// Two byte address + x register (e.g. ADC $5f70,X)
    AbsoluteX,
    /// Two byte address + y register (e.g. ADC $5f70,Y)
    AbsoluteY,
    /// Immediate operand (e.g. ADC #$64)
    Immediate,
    /// Signed relative offset from the next instruction (e.g. BNE label, where label is in the range +129/-126)
    Relative,
    /// Two byte address to memory location holding a two byte address
    /// (e.g. JMP ($f0f0))
    Indirect,
    /// Single byte address + x register point to memory location holding a two byte address,
    /// first address wraps around to zero page (e.g. ADC ($34,X))
    IndexedIndirect,
    /// Single byte address pointing to two byte address, add y register to two byte address
    /// (e.g. ADC ($f0),Y)
    IndirectIndexed,
}

/// Flags in the P register
#[derive(Debug)]
enum Flags {
    Carry = 0x01,
    Zero = 0x02,
    InterruptDisable = 0x04,
    Decimal = 0x08,
    Overflow = 0x40,
    Negative = 0x80,
}

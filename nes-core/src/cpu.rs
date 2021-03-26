use crate::{cpu_ops::{CPU_OPS, CpuOp}, memory::Memory};

pub const CPU_CLOCK_DIV: u64 = 12;

pub struct Cpu {
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_pc: u16,
    reg_s: u8,
    reg_p: u8,

    opmap: [CpuOp; 0x100],

    master_clock: u64,
}

impl Cpu {
    pub fn new() -> Self {
        let mut opmap = [CpuOp{ name: "???", opcode: 0x00, addr_mode: AddressingMode::Implicit, func: Self::op_invalid}; 0x100];

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

            opmap,

            master_clock: 0
        }
    }

    /// Resets the CPU to the following state
    /// - P: InterruptDisable
    /// - A, X, Y: 0
    /// - S: 0xFD
    /// - PC: loaded from reset vector (0xFFFC)
    ///
    /// The reset will take 7 cpu cycles
    pub fn reset(&mut self, memory: &mut dyn Memory) {
        self.master_clock = 7 * CPU_CLOCK_DIV;

        self.reg_p = Flags::InterruptDisable as u8;
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_s = 0xFD;
        
        let pc_low = memory.cpu_load8(0xFFFC);
        let pc_high = memory.cpu_load8(0xFFFD);
        self.reg_pc = ((pc_high as u16) << 8) | (pc_low as u16);
    }

    /// Performs a single CPU Instruction
    pub fn execute_single_instruction(&mut self, memory: &mut dyn Memory) {
        // cycle 0: load opcode, increment PC
        let opcode = memory.cpu_load8(self.reg_pc);
        let op = self.opmap[opcode as usize];

        println!("{:0>4X}  {}  A:{:0>2X} X:{:0>2X} Y:{:0>2X} P:{:0>2X} SP:{:0>2X}  CYC:{}", self.reg_pc, op.name, self.reg_a, self.reg_x, self.reg_y, self.reg_p | 0x20, self.reg_s, self.master_clock / CPU_CLOCK_DIV as u64);
    
        self.reg_pc += 1;
        self.master_clock += CPU_CLOCK_DIV;

        (op.func)(self, op.addr_mode, memory);
    }

    /// Instruction that is executed when an unofficial opcode is encountered
    pub(crate) fn op_invalid(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.op_nop(addr_mode, memory)
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
    fn get_operand_addr(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory, is_read: bool) -> u16 {
        match addr_mode {
            AddressingMode::Implicit => {
                // cycle 1: read next instruction byte and throw it away
                memory.cpu_load8(self.reg_pc);
                self.master_clock += CPU_CLOCK_DIV;
                0
            }
            AddressingMode::ZeroPage => {
                // cycle 1: load immediate 1 byte address
                let arg = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;
                arg as u16
            }
            AddressingMode::ZeroPageX => {
                // cycle 1: load immediate 1 byte address
                let mut arg = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: dummy read from unindexed address, add X to address
                memory.cpu_load8(arg as u16);
                self.master_clock += CPU_CLOCK_DIV;
                // add x
                arg = arg.wrapping_add(self.reg_x);
                arg as u16
            }
            AddressingMode::ZeroPageY => {
                // cycle 1: load immediate 1 byte address
                let mut arg = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: dummy read from unindexed address, add Y to address
                memory.cpu_load8(arg as u16);
                self.master_clock += CPU_CLOCK_DIV;
                // add y
                arg = arg.wrapping_add(self.reg_y);
                arg as u16
            }
            AddressingMode::Absolute => {
                // cycle 1: load low address byte
                let addr_low = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: load high address byte
                let addr_high = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                let addr = ((addr_high as u16) << 8) | (addr_low as u16);
                addr
            }
            AddressingMode::AbsoluteX => {
                // cycle 1: load low addr byte
                let mut base_addr = memory.cpu_load8(self.reg_pc) as u16;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: load high addr byte
                base_addr |= (memory.cpu_load8(self.reg_pc) as u16) << 8;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                let real_addr = base_addr + self.reg_x as u16;

                // write and read-modify-write instructions always read the unfixed effective addr once without using the value,
                // read instructions only have this wasted read on a page crossing
                if !is_read || ((real_addr & 0xFF00) != (base_addr & 0xFF00)) {
                    memory.cpu_load8((base_addr & 0xFF00) | (real_addr & 0x00FF));
                    self.master_clock += CPU_CLOCK_DIV;
                }

                real_addr
            }
            AddressingMode::AbsoluteY => {
                // cycle 1: load low addr byte
                let mut base_addr = memory.cpu_load8(self.reg_pc) as u16;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: load high addr byte
                base_addr |= (memory.cpu_load8(self.reg_pc) as u16) << 8;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                let real_addr = base_addr.wrapping_add(self.reg_y as u16);

                // write and read-modify-write instructions always read the unfixed effective addr once without using the value,
                // read instructions only have this wasted read on a page crossing
                if !is_read || ((real_addr & 0xFF00) != (base_addr & 0xFF00)) {
                    memory.cpu_load8((base_addr & 0xFF00) | (real_addr & 0x00FF));
                    self.master_clock += CPU_CLOCK_DIV;
                }

                real_addr
            }
            AddressingMode::Immediate | AddressingMode::Relative => {
                // cycle 1: read immediate operand
                let addr = self.reg_pc;
                self.reg_pc = self.reg_pc.wrapping_add(1);
                // note: no clock increment because whichever instruction uses this function
                // will load the value on its own
                //self.master_clock += CPU_CLOCK_DIV;

                addr
            }
            AddressingMode::Indirect => {
                // cycle 1: load ptr low
                let ptr_low = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: load ptr high
                let ptr_high = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 3: load addr low
                let addr_low = memory.cpu_load8(((ptr_high as u16) << 8) | (ptr_low as u16));
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 4: load addr high
                // note: if ptr_low is 0xFF, no page crossing will be handled
                let addr_high = memory.cpu_load8(((ptr_high as u16) << 8) | (ptr_low.wrapping_add(1) as u16));
                self.master_clock += CPU_CLOCK_DIV;
                
                ((addr_high as u16) << 8) | (addr_low as u16)
            }
            AddressingMode::IndexedIndirect => {
                // cycle 1: load ptr
                let mut ptr = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: dummy read address, add X
                memory.cpu_load8(ptr as u16);
                ptr = ptr.wrapping_add(self.reg_x);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 3: load addr low
                let addr_low = memory.cpu_load8(ptr as u16);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 4: load addr high
                // note: no page crossing will be handled
                let addr_high = memory.cpu_load8(ptr.wrapping_add(1) as u16);
                self.master_clock += CPU_CLOCK_DIV;

                ((addr_high as u16) << 8) | (addr_low as u16)
            }
            AddressingMode::IndirectIndexed => {
                // cycle 1: load ptr
                let ptr = memory.cpu_load8(self.reg_pc);
                self.reg_pc = self.reg_pc.wrapping_add(1);
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 2: load addr low
                let mut base_addr = memory.cpu_load8(ptr as u16) as u16;
                self.master_clock += CPU_CLOCK_DIV;

                // cycle 3: load addr high
                base_addr |= (memory.cpu_load8(ptr.wrapping_add(1) as u16) as u16) << 8;
                self.master_clock += CPU_CLOCK_DIV;

                let real_addr = base_addr.wrapping_add(self.reg_y as u16);

                // write and read-modify-write instructions always do a useless read of the unfixed addr,
                // read instructions only when a page is crossed by adding y
                if !is_read || ((real_addr & 0xFF00) != (base_addr & 0xFF00)) {
                    memory.cpu_load8((base_addr & 0xFF00) | (real_addr & 0x00FF));
                    self.master_clock += CPU_CLOCK_DIV;
                }

                real_addr
            }
        }
    }

    pub(crate) fn op_adc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);

        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        let carry_in: u16 = if self.get_flag(Flags::Carry) { 1 } else { 0 };

        let res = (op as u16).wrapping_add(self.reg_a as u16).wrapping_add(carry_in);

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        let overflow = (!(self.reg_a ^ op)) & (self.reg_a ^ (res & 0xFF) as u8) & 0x80;
        self.set_flag(Flags::Overflow, overflow != 0);

        self.reg_a = (res & 0xFF) as u8;

        0
    }

    pub(crate) fn op_and(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);

        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        let res = self.reg_a & op;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = res;

        0
    }

    pub(crate) fn op_asl_a(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        let res = (self.reg_a as u16) << 1;

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = (res & 0xFF) as u8;
        0
    }

    pub(crate) fn op_asl_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);

        // read operand
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        // dummy write value back
        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let res = (op as u16) << 1;

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        // write result
        memory.cpu_store8(op_addr, (res & 0xFF) as u8);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    /// Performs a relative branch with `op` as signed 8-Bit Offset
    /// # Cycles
    /// - A branch instruction that does not branch takes 2 Cycles
    /// - If a branch is taken, add one cycle
    /// - If the branch crosses a page (e.g. 0x01xx -> 0x02xx), add another cycle
    fn relative_branch(&mut self, op: u8, memory: &mut dyn Memory) -> u8 {
        // on a taken branch, the next instruction is read and discarded
        memory.cpu_load8(self.reg_pc);
        self.master_clock += CPU_CLOCK_DIV;

        let mut offs = op as u16;
        // perform sign extension
        if (offs & 0x80) != 0 {
            offs |= 0xFF00;
        }

        let new_pc = self.reg_pc.wrapping_add(offs);

        if (new_pc & 0xFF00) != (self.reg_pc & 0xFF00) {
            // on page cross add another dummy read at the unfixed new pc
            memory.cpu_load8((self.reg_pc & 0xFF00) | (new_pc & 0x00FF));
            self.master_clock += CPU_CLOCK_DIV;
        }

        self.reg_pc = new_pc;
        0
    }

    pub(crate) fn op_bcc(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if !self.get_flag(Flags::Carry) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_bcs(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if self.get_flag(Flags::Carry) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_beq(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if self.get_flag(Flags::Zero) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_bit(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        let res = self.reg_a & op;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Overflow, (op & 0x40) != 0);
        self.set_flag(Flags::Negative, (op & 0x80) != 0);

        0
    }

    pub(crate) fn op_bmi(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if self.get_flag(Flags::Negative) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_bne(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if !self.get_flag(Flags::Zero) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_bpl(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if !self.get_flag(Flags::Negative) {
            self.relative_branch(op, memory)
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

        let vect_low = memory.cpu_load8(0xFFFE);
        self.master_clock += CPU_CLOCK_DIV;

        let vect_high = memory.cpu_load8(0xFFFF);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_pc = ((vect_high as u16) << 8) | (vect_low as u16);
        0
    }

    pub(crate) fn op_bvc(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if !self.get_flag(Flags::Overflow) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_bvs(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(AddressingMode::Relative, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        if self.get_flag(Flags::Overflow) {
            self.relative_branch(op, memory)
        } else {
            0
        }
    }

    pub(crate) fn op_clc(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::Carry, false);
        0
    }

    pub(crate) fn op_cld(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::Decimal, false);
        0
    }

    pub(crate) fn op_cli(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::InterruptDisable, false);
        0
    }

    pub(crate) fn op_clv(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::Overflow, false);
        0
    }

    pub(crate) fn op_cmp(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.set_flag(Flags::Carry, self.reg_a >= op);
        self.set_flag(Flags::Zero, self.reg_a == op);

        let tmp = (self.reg_a as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        0
    }

    pub(crate) fn op_cpx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.set_flag(Flags::Carry, self.reg_x >= op);
        self.set_flag(Flags::Zero, self.reg_x == op);

        let tmp = (self.reg_x as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        0
    }

    pub(crate) fn op_cpy(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.set_flag(Flags::Carry, self.reg_y >= op);
        self.set_flag(Flags::Zero, self.reg_y == op);

        let tmp = (self.reg_y as u16).wrapping_sub(op as u16);
        self.set_flag(Flags::Negative, (tmp & 0x80) != 0);

        0
    }

    pub(crate) fn op_dec(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let res = op.wrapping_sub(1);

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.cpu_store8(op_addr, res);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_dex(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_x = self.reg_x.wrapping_sub(1);

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_dey(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_y = self.reg_y.wrapping_sub(1);

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_eor(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_a ^= op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_inc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let res = op.wrapping_add(1);

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.cpu_store8(op_addr, res);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_inx(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);
        
        self.reg_x = self.reg_x.wrapping_add(1);

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_iny(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);
        
        self.reg_y = self.reg_y.wrapping_add(1);

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_jmp(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);

        self.reg_pc = op_addr;

        0
    }

    pub(crate) fn op_jsr(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        // note: no self.get_operand_addr here because this instruction
        // has an unusual cycle layout that does not match absolute addressing
        let addr_low = memory.cpu_load8(self.reg_pc);
        self.reg_pc = self.reg_pc.wrapping_add(1);
        self.master_clock += CPU_CLOCK_DIV;

        // dummy read from stack
        memory.cpu_load8(0x0100 | self.reg_s as u16);
        self.master_clock += CPU_CLOCK_DIV;

        self.push((self.reg_pc >> 8) as u8, memory);
        self.push((self.reg_pc & 0xFF) as u8, memory);

        let addr_high = memory.cpu_load8(self.reg_pc);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_pc = ((addr_high as u16) << 8) | (addr_low as u16);

        0
    }

    pub(crate) fn op_lda(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_a = op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_ldx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_x = op;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_ldy(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_y = op;

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_lsr_a(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        let res = self.reg_a.wrapping_shr(1);

        self.set_flag(Flags::Carry, (self.reg_a & 0x01) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        self.reg_a = res;
        0
    }

    pub(crate) fn op_lsr_m(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let res = op.wrapping_shr(1);

        self.set_flag(Flags::Carry, (op & 0x01) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.cpu_store8(op_addr, res);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_nop(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        0
    }

    pub(crate) fn op_ora(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        self.reg_a |= op;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
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
        memory.cpu_store8(addr, val);
        self.master_clock += CPU_CLOCK_DIV;
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
        let res = memory.cpu_load8(addr);
        self.master_clock += CPU_CLOCK_DIV;

        res
    }

    pub(crate) fn op_pha(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.push(self.reg_a, memory);
        0
    }

    pub(crate) fn op_php(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        let val = self.reg_p | 0x30;
        self.push(val, memory);
        0
    }

    pub(crate) fn op_pla(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        memory.cpu_load8(0x0100 | (self.reg_s as u16));
        self.master_clock += CPU_CLOCK_DIV;

        let val = self.pull(memory);
        self.reg_a = val;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_plp(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        memory.cpu_load8(0x0100 | (self.reg_s as u16));
        self.master_clock += CPU_CLOCK_DIV;

        let val = self.pull(memory);
        self.reg_p = val & 0xCF;

        0
    }

    pub(crate) fn op_rol_a(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

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
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let mut res = (op as u16) << 1;
        if self.get_flag(Flags::Carry) {
            res |= 0x01;
        }

        self.set_flag(Flags::Carry, (res & 0x100) != 0);

        let res = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.cpu_store8(op_addr, res);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_ror_a(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

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
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        let op = memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        memory.cpu_store8(op_addr, op);
        self.master_clock += CPU_CLOCK_DIV;

        let mut res = op.wrapping_shr(1);
        if self.get_flag(Flags::Carry) {
            res |= 0x80;
        }

        self.set_flag(Flags::Carry, (op & 0x01) != 0);

        let res = (res & 0xFF) as u8;

        self.set_flag(Flags::Zero, res == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        memory.cpu_store8(op_addr, res);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_rti(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        memory.cpu_load8(0x0100 | (self.reg_s as u16));
        self.master_clock += CPU_CLOCK_DIV;

        let p = self.pull(memory);
        let ret_addr_low = self.pull(memory);
        let ret_addr_high = self.pull(memory);

        let ret_addr = ((ret_addr_high as u16) << 8) | (ret_addr_low as u16);

        self.reg_p = p & 0xCF;
        self.reg_pc = ret_addr;

        0
    }

    pub(crate) fn op_rts(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        memory.cpu_load8(0x0100 | (self.reg_s as u16));
        self.master_clock += CPU_CLOCK_DIV;

        let ret_addr_low = self.pull(memory);
        let ret_addr_high = self.pull(memory);

        let ret_addr = ((ret_addr_high as u16) << 8) | (ret_addr_low as u16);

        self.reg_pc = ret_addr.wrapping_add(1);

        memory.cpu_load8(ret_addr);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_sbc(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, true);
        let op = !memory.cpu_load8(op_addr);
        self.master_clock += CPU_CLOCK_DIV;

        let carry_in: u16 = self.get_flag(Flags::Carry) as u16;

        let res = (op as u16).wrapping_add(self.reg_a as u16).wrapping_add(carry_in);

        self.set_flag(Flags::Carry, (res & 0x100) != 0);
        self.set_flag(Flags::Zero, (res & 0xFF) == 0);
        self.set_flag(Flags::Negative, (res & 0x80) != 0);

        let overflow = (!(self.reg_a ^ op)) & (self.reg_a ^ (res & 0xFF) as u8) & 0x80;
        self.set_flag(Flags::Overflow, overflow != 0);

        self.reg_a = (res & 0xFF) as u8;

        0
    }

    pub(crate) fn op_sec(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::Carry, true);
        0
    }

    pub(crate) fn op_sed(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::Decimal, true);
        0
    }

    pub(crate) fn op_sei(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.set_flag(Flags::InterruptDisable, true);
        0
    }

    pub(crate) fn op_sta(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        
        memory.cpu_store8(op_addr, self.reg_a);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_stx(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        
        memory.cpu_store8(op_addr, self.reg_x);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_sty(&mut self, addr_mode: AddressingMode, memory: &mut dyn Memory) -> u8 {
        let op_addr = self.get_operand_addr(addr_mode, memory, false);
        
        memory.cpu_store8(op_addr, self.reg_y);
        self.master_clock += CPU_CLOCK_DIV;

        0
    }

    pub(crate) fn op_tax(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_x = self.reg_a;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_tay(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_y = self.reg_a;

        self.set_flag(Flags::Zero, self.reg_y == 0);
        self.set_flag(Flags::Negative, (self.reg_y & 0x80) != 0);

        0
    }

    pub(crate) fn op_tsx(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_x = self.reg_s;

        self.set_flag(Flags::Zero, self.reg_x == 0);
        self.set_flag(Flags::Negative, (self.reg_x & 0x80) != 0);

        0
    }

    pub(crate) fn op_txa(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_a = self.reg_x;

        self.set_flag(Flags::Zero, self.reg_a == 0);
        self.set_flag(Flags::Negative, (self.reg_a & 0x80) != 0);

        0
    }

    pub(crate) fn op_txs(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

        self.reg_s = self.reg_x;

        0
    }

    pub(crate) fn op_tya(&mut self, _: AddressingMode, memory: &mut dyn Memory) -> u8 {
        self.get_operand_addr(AddressingMode::Implicit, memory, false);

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

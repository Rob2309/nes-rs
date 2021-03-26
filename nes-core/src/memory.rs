
/// Interface used by the CPU for memory accesses
pub trait Memory {
    fn cpu_load8(&mut self, addr: u16) -> u8;
    fn cpu_store8(&mut self, addr: u16, val: u8);

    fn ppu_load8(&mut self, addr: u16) -> u8;
    fn ppu_store8(&mut self, addr: u16, val: u8);
}

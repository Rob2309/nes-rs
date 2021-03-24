
/// Interface used by the CPU for memory accesses
pub trait Memory {
    fn load8(&mut self, addr: u16) -> u8;
    fn load16(&mut self, addr: u16) -> u16;

    fn store8(&mut self, addr: u16, val: u8);
}

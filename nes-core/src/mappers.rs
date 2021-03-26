/// Interface used to load data into a Mapper by the INES Loader
pub trait Mapper {
    /// Called by the INES loader to set the PRG ROM data
    /// 
    /// `prg_rom.len()` will always be a multiple of 16KB/0x4000
    fn load_prg_rom(&mut self, prg_rom: &[u8]);

    /// Called by the INES loader to set the CHR ROM data
    /// 
    /// `chr_rom.len()` will always be a multiple of 8KB/0x2000
    fn load_chr_rom(&mut self, chr_rom: &[u8]);

    /// Called by the INES loader to inform the Mapper how much PRG RAM the
    /// given INES file requested
    fn set_ram_size(&mut self, size: u16);

    /// This function should overwrite a memory cell in PRG ROM without causing any side effects
    /// (e.g. bank switching)
    /// 
    /// Only used for debugging purposes (e.g. forcing the reset vector to a different value)
    fn overwrite_prg_rom(&mut self, addr: u16, val: u8);

    fn cpu_load8(&mut self, addr: u16) -> u8;
    fn cpu_store8(&mut self, addr: u16, val: u8);

    fn ppu_load8(&mut self, addr: u16) -> u8;
    fn ppu_store8(&mut self, addr: u16, val: u8);
}

mod mapper000;
pub use mapper000::Mapper000;
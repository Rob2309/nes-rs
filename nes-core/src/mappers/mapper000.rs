use crate::memory::Memory;

use super::Mapper;


/// NROM Mapper (http://wiki.nesdev.com/w/index.php/NROM)
/// 
/// INES Mapper ID: 0
/// 
/// - PRG ROM: 16 or 32 KB at 0x8000 as necessary mirrored to 0xFFFF, no bank switching
/// - CHR ROM: 8 KB, no bank switching
/// - Nametable mirroring: fixed vertical or horizontal
pub struct Mapper000 {
    cpu_ram: [u8; 0x800],
    prg_rom: [u8; 0x8000],
    prg_rom_mask: u16,
    chr_rom: [u8; 0x2000],
}

impl Mapper000 {
    pub fn new() -> Self {
        Self {
            cpu_ram: [0; 0x800],
            prg_rom: [0; 0x8000],
            prg_rom_mask: 0,
            chr_rom: [0; 0x2000],
        }
    }
}

impl Mapper for Mapper000 {
    fn load_prg_rom(&mut self, prg_rom: &[u8]) {
        let prg_rom_size = self.prg_rom.len().min(prg_rom.len());
        self.prg_rom[..prg_rom_size].copy_from_slice(&prg_rom[..prg_rom_size]);
        self.prg_rom_mask = if prg_rom.len() <= 0x4000 { 0x3FFF } else { 0x7FFF }
    }

    fn load_chr_rom(&mut self, chr_rom: &[u8]) {
        assert!(chr_rom.len() <= 0x2000);
        self.chr_rom[..chr_rom.len()].copy_from_slice(chr_rom);
    }

    fn set_ram_size(&mut self, size: u16) {
        
    }

    fn overwrite_prg_rom(&mut self, addr: u16, val: u8) {
        self.prg_rom[(addr & self.prg_rom_mask) as usize] = val;
    }
}

impl Memory for Mapper000 {
    fn cpu_load8(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.cpu_ram[(addr & 0x7FF) as usize]
        } else if addr >= 0x8000 {
            self.prg_rom[(addr & self.prg_rom_mask) as usize]
        } else {
            0
        }
    }

    fn cpu_store8(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.cpu_ram[(addr & 0x7FF) as usize] = val;
        }
    }

    fn ppu_load8(&mut self, addr: u16) -> u8 {
        todo!()
    }

    fn ppu_store8(&mut self, addr: u16, val: u8) {
        todo!()
    }
}

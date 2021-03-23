use std::fs;

use nes_core::{cpu::Cpu, memory::Memory};


struct Mapper000 {
    cpu_ram: [u8; 0x800],
    cart_rom: [u8; 0x4000],
}

impl Mapper000 {
    pub fn new() -> Self {
        Self{
            cpu_ram: [0; 0x800],
            cart_rom: [0; 0x4000],
        }
    }
}

impl Memory for Mapper000 {
    fn load8(&mut self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.cpu_ram[(addr & 0x7FF) as usize]
        } else if addr >= 0x8000 {
            self.cart_rom[(addr & 0x3FFF) as usize]
        } else {
            0
        }
    }

    fn load16(&mut self, addr: u16) -> u16 {
        let low = self.load8(addr);
        let high = self.load8(addr.wrapping_add(1));
        ((high as u16) << 8) | (low as u16)
    }

    fn store8(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            self.cpu_ram[(addr & 0x7FF) as usize] = val;
        } else if addr >= 0x8000 {
            self.cart_rom[(addr & 0x3FFF) as usize] = val;
        }
    }
}

fn load_test(cart_rom: &mut [u8; 0x4000]) {
    let data = fs::read("roms/nestest.nes").unwrap();
    cart_rom.copy_from_slice(&data[16..0x4010]);
}

fn main() {
    let mut cpu = Cpu::new();
    let mut mapper = Mapper000::new();

    load_test(&mut mapper.cart_rom);

    mapper.store8(0xFFFC, 0x00);
    mapper.store8(0xFFFD, 0xC0);

    cpu.reset(&mut mapper);

    for i in 0..26555 {
        cpu.cycle(&mut mapper);
    }
}

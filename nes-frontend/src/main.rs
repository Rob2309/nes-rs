use std::fs;

use nes_core::{cpu::Cpu, mappers::{Mapper, Mapper000}};

fn create_mapper(id: u8) -> Box<dyn Mapper> {
    match id {
        0x00 => { Box::new(Mapper000::new()) }
        _ => { panic!("No mapper with id {}", id) }
    }
}

fn load_ines(path: &str) -> Box<dyn Mapper> {
    let data = fs::read(path).unwrap();

    if data[0] != b'N' || data[1] != b'E' || data[2] != b'S' || data[3] != 0x1A {
        panic!("Invalid INES Magic");
    }

    let prg_rom_size = data[4] as usize* 0x4000;
    let chr_rom_size = data[5] as usize * 0x2000;

    let mapper_id = ((data[6] & 0xF0) >> 4) | (data[7] & 0xF0);

    let mut mapper = create_mapper(mapper_id);

    mapper.load_prg_rom(&data[16..16+prg_rom_size]);
    mapper.load_chr_rom(&data[16+prg_rom_size..16+prg_rom_size+chr_rom_size]);

    mapper
}

fn main() {
    let mut cpu = Cpu::new();

    let mut mapper = load_ines("roms/nestest.nes");

    mapper.overwrite_prg_rom(0xFFFC, 0x00);
    mapper.overwrite_prg_rom(0xFFFD, 0xC0);

    cpu.reset(mapper.as_mut());

    for _ in 0..9000 {
        cpu.execute_single_instruction(mapper.as_mut());
    }
}

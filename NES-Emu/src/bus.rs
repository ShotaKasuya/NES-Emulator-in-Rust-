use crate::{ppu::NesPPU, rom::Rom};

pub struct Bus {
  cpu_vram: [u8; 0x800],
  prg_rom: Vec<u8>,
  ppu: NesPPU,
  cycles: usize,
}

impl Bus {
  pub fn new(rom: Rom) -> Self {
    let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);
    Bus {
      cpu_vram: [0; 0x800],
      prg_rom: rom.prg_rom,
      ppu: ppu,
      cycles: 0,
    }
  }

  fn read_prg_rom(&self, mut addr: u16) -> u8 {
    addr -= 0x8000;
    if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      // mirror if needed
      addr = addr % 0x4000;
    }
    self.prg_rom[addr as usize]
  }
  pub fn tick(&mut self, cycle: u8) {
    self.cycles += cycle as usize;
    self.ppu.tick(cycle * 3);
  }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub trait Mem {
  fn mem_read(&self, addr: u16) -> u8;
  fn mem_write(&mut self, addr: u16, data: u8);
}

impl Mem for Bus {
  fn mem_read(&self, addr: u16) -> u8 {
    match addr {
      RAM..=RAM_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirror_down_addr as usize]
      }
      0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
        panic!("Attempt to read from write-only PPU address {:X}", addr);
      }
      0x2007 => self.ppu.read_data(),
      0x2008..=PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0010_0000_0000_0111;
        self.mem_read(mirror_down_addr)
      }
      0x8000..=0xFFFF => self.read_prg_rom(addr),
      _ => {
        println!("Ignoreing mem access at {}", addr);
        0
      }
    }
  }

  fn mem_write(&mut self, addr: u16, data: u8) {
    match addr {
      RAM..=RAM_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirror_down_addr as usize] = data;
      }
      0x2000 => self.ppu.write_to_ctrl(data),
      0x2006 => self.ppu.write_to_ppu_addr(data),
      0x2007 => self.ppu.write_to_data(data),
      0x2008..=PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0010_0000_0000_0111;
        self.mem_write(mirror_down_addr, data);
      }
      0x8000..=0xFFFF => {
        panic!("Attempt to write to Cartridge ROM space")
      }
      _ => {
        println!("Ignoring mem write-access at {}", addr);
      }
    }
  }
}

use log::{error, trace, warn};

use crate::{apu::NesAPU, joypad::Joypad, ppu::NesPPU, rom::Rom};

pub struct Bus<'call> {
  cpu_vram: [u8; 0x800],
  prg_rom: Vec<u8>,
  ppu: NesPPU,
  apu: NesAPU,
  cycles: usize,
  joypad1: Joypad,
  joypad2: Joypad,
  gameloop_callback: Box<dyn FnMut(&NesPPU, &mut Joypad) + 'call>,
}

impl<'a> Bus<'a> {
  pub fn new<'call, F>(rom: Rom, apu: NesAPU, gameloop_callback: F) -> Bus<'call>
  where
    F: FnMut(&NesPPU, &mut Joypad) + 'call,
  {
    let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);
    Bus {
      cpu_vram: [0; 0x800],
      prg_rom: rom.prg_rom,
      ppu: ppu,
      apu: apu,
      joypad1: Joypad::new(),
      joypad2: Joypad::new(),
      cycles: 0,
      gameloop_callback: Box::from(gameloop_callback),
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

    let nmi_before = self.ppu.nmi_interrupt.is_some();
    self.ppu.tick(cycle * 3);
    let nmi_after = self.ppu.nmi_interrupt.is_some();

    if !nmi_before && nmi_after {
      (self.gameloop_callback)(&self.ppu, &mut self.joypad1);
    }
  }

  pub fn poll_nmi_status(&mut self) -> Option<i32> {
    if self.ppu.clear_nmi_interrupt {
      self.ppu.clear_nmi_interrupt = false;
      self.ppu.nmi_interrupt = None;
      return None;
    }
    let res = self.ppu.nmi_interrupt;
    self.ppu.nmi_interrupt = None;
    res
  }
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub trait Mem {
  fn mem_read(&mut self, addr: u16) -> u8;
  fn mem_write(&mut self, addr: u16, data: u8);
}

impl Mem for Bus<'_> {
  fn mem_read(&mut self, addr: u16) -> u8 {
    match addr {
      RAM..=RAM_MIRRORS_END => {
        //                            0x07FF
        let mirror_down_addr = addr & 0b0000_0111_1111_1111;
        let v = self.cpu_vram[mirror_down_addr as usize];
        trace!(
          "RAM READ {:04X} => {:04X} ({:02X})",
          addr,
          mirror_down_addr,
          v
        );

        v
      }
      0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
        warn!("Attempt to read from write-only PPU address {:X}", addr);
        0
      }
      0x2002 => self.ppu.read_status(),
      0x2004 => self.ppu.read_oam_data(),
      0x2007 => self.ppu.read_data(),
      0x2008..=PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0010_0000_0000_0111;
        self.mem_read(mirror_down_addr)
      }
      0x4016 => self.joypad1.read(),
      0x4017 => self.joypad2.read(),
      PRG_ROM..=PRG_ROM_END => self.read_prg_rom(addr),
      _ => {
        warn!("Ignoreing mem access at {}", addr);
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
      0x2001 => self.ppu.write_to_mask(data),
      0x2002 => self.ppu.write_to_status(data),
      0x2003 => self.ppu.write_to_oam_addr(data),
      0x2004 => self.ppu.write_to_oam_data(data),
      // TODO 0x2005 => scroll
      0x2005 => {
        self.ppu.write_to_scroll(data);
      }
      0x2006 => self.ppu.write_to_ppu_addr(data),
      0x2007 => self.ppu.write_to_data(data),
      0x2008..=PPU_REGISTERS_MIRRORS_END => {
        let mirror_down_addr = addr & 0b0010_0000_0000_0111;
        self.mem_write(mirror_down_addr, data);
      }
      0x4000..=0x4003 => {
        self.apu.write_1ch(addr, data);
      }
      0x4004..=0x4007 => {
        self.apu.write_2ch(addr, data);
      }
      0x4008 | 0x400A | 0x400B => {
        self.apu.write_3ch(addr, data);
      }
      0x400C | 0x400E | 0x400F => {
        self.apu.write_4ch(addr, data);
      }
      0x4010..=0x4013 | 0x4015 => {
        // TODO APU DMCch
      }
      // TODO 0x4014 => OAM DMA
      0x4014 => {
        // $XX を書き込むと、256 バイトのデータが CPU ページ $XX00 ～ $XXFF から内部 PPU OAM にアップロードされます。
        // このページは通常、内部 RAM (通常は $0200 ～ $02FF) にありますが、カートリッジ RAM または ROM も使用できます。

        let mut values: [u8; 256] = [0; 256];
        for i in 0x00..0xFF {
          values[i] = self.mem_read((data as u16) << 8 | i as u16);
        }
        self.ppu.write_to_oam_dma(values);

        // OMA DMAにはCPUクロックが513サイクルかかる
        for _ in 0..513 {
          self.ppu.tick(1);
        }
      }
      0x4016 => {
        self.joypad1.write(data);
      }
      0x4017 => {
        self.joypad2.write(data);
      }
      PRG_ROM..=PRG_ROM_END => {
        warn!("Attempt to write to Cartridge ROM space");
      }
      _ => {
        error!("Ignoring mem write-access at {:X}", addr);
      }
    }
  }
}

use crate::rom::Mirroring;
use bitflags::bitflags;

pub struct NesPPU {
  pub chr_rom: Vec<u8>,
  pub palette_table: [u8; 32],
  pub vram: [u8; 2048],

  pub oam_addr: u8,
  pub oam_data: [u8; 256],

  pub mirroring: Mirroring,
  addr: AddrResister,        // 0x2006 (0x2007)
  pub ctrl: ControlRegister, // 0x2000
  internal_data_buf: u8,

  // TODO mask 0x2001
  mask: MaskRegister,
  // TODO status 0x2002
  status: StatusRegister,
  // TODO OAM Address 0x2003, 0x2004
  // TODO scroll 0x2005
  pub scroll: ScrollRegister,
  // TODO OAM DMA 0x4015
  scanline: u16,
  cycles: usize,
  pub nmi_interrupt: Option<i32>,
}

impl NesPPU {
  pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
    NesPPU {
      chr_rom: chr_rom,
      mirroring: mirroring,
      vram: [0; 2048],
      oam_addr: 0,
      oam_data: [0; 64 * 4],
      palette_table: [0; 32],
      ctrl: ControlRegister::new(),
      addr: AddrResister::new(),
      status: StatusRegister::new(),
      mask: MaskRegister::new(),
      scroll: ScrollRegister::new(),
      internal_data_buf: 0,
      scanline: 0,
      cycles: 0,
      nmi_interrupt: None,
    }
  }

  pub fn write_to_ppu_addr(&mut self, value: u8) {
    self.addr.update(value);
  }

  pub fn write_to_data(&mut self, value: u8) {
    let addr = self.addr.get();
    self.increment_vram_addr();

    match addr {
      0..=0x1FFF => {
        // panic!(
        //   "addr space 0x0000..0x1FFF is not expected to be used, requested = {:X} ",
        //   addr,
        // )
      }
      0x2000..=0x2FFF => {
        self.vram[self.mirror_vram_addr(addr) as usize] = value;
      }
      0x3000..=0x3EFF => {
        // panic!(
        //   "addr space 0x3000..0x3EFF is not expected to be used, requested = {}",
        //   addr
        // )
      }
      0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize] = value,
      _ => panic!("unexpected access to mittord space {}", addr),
    }
  }

  pub fn write_to_ctrl(&mut self, value: u8) {
    let before_nmi_status = self.ctrl.generate_vblank_nmi();

    self.ctrl.update(value);
    if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
      self.nmi_interrupt = Some(1);
    }
  }

  pub fn read_status(&mut self) -> u8 {
    // スクロール($2005)PPU_STATUSを読み取ってアドレスラッチをリセットしたあと
    self.scroll.reset();
    self.status.bits()
  }

  pub fn write_to_status(&mut self, value: u8) {
    self.status.update(value);
  }

  pub fn write_to_mask(&mut self, value: u8) {
    self.mask.update(value);
  }

  pub fn write_to_oam_addr(&mut self, value: u8) {
    self.oam_addr = value;
  }

  pub fn write_to_oam_data(&mut self, value: u8) {
    self.oam_data[self.oam_addr as usize] = value;
    self.oam_addr = self.oam_addr.wrapping_add(1);
  }

  pub fn read_oam_data(&self) -> u8 {
    self.oam_data[self.oam_addr as usize]
  }

  pub fn write_to_oam_dma(&mut self, values: [u8; 256]) {
    self.oam_data = values;
  }

  pub fn write_to_scroll(&mut self, value: u8) {
    self.scroll.set(value);
  }

  fn increment_vram_addr(&mut self) {
    self.addr.increment(self.ctrl.vram_addr_increment());
  }

  pub fn read_data(&mut self) -> u8 {
    let addr = self.addr.get();
    self.increment_vram_addr();

    match addr {
      0..=0x1FFF => {
        let result = self.internal_data_buf;
        self.internal_data_buf = self.chr_rom[addr as usize];
        result
      }
      0x2000..=0x2FFF => {
        let result = self.internal_data_buf;
        self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      }
      0x3000..=0x3EFF => panic!(
        "addr space 0x3000..0x3EFF is not expected to be used, requested = {}",
        addr
      ),
      0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
      _ => panic!("unexpected access to mittord space {}", addr),
    }
  }

  pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
    let mirrored_vram = addr & 0b10_1111_1111_1111;
    let vram_index = mirrored_vram - 0x2000;
    let name_table = vram_index / 0x400;

    match (&self.mirroring, name_table) {
      (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
      (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
      (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
      (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
      _ => vram_index,
    }
  }

  pub fn tick(&mut self, cycles: u8) -> bool {
    self.cycles += cycles as usize;
    if self.cycles >= 341 {
      // if (self.status.is_sprite_zero_hit(self.cycles)) {
      //   self.status.set_sprite_zero_hit(true);
      // }
      self.cycles = self.cycles - 341;
      self.scanline += 1;

      if self.scanline == 241 {
        // self.status.set_sprite_zero_hit(false);
        if self.ctrl.generate_vblank_nmi() {
          self.status.set_vblank_status(true);
          // todo!("Should trigger NMI interrupt")
          self.nmi_interrupt = Some(1);
        }
      }

      if self.scanline >= 262 {
        self.scanline = 0;
        // self.set_sprite_zero_hit(false);
        self.status.reset_vblank_status();
        // FIXME..
        // self.nmi_interrupt = None;
        return true;
      }
    }
    return false;
  }

  fn is_sprite_zero_hit(&self, cycle: usize) -> bool {
    let y = self.oam_data[0] as usize;
    let x = self.oam_data[3] as usize;
    (y == self.scanline as usize) && x <= cycle && self.mask.show_sprites()
  }
}

pub struct AddrResister {
  value: (u8, u8),
  hi_ptr: bool,
}
impl AddrResister {
  pub fn new() -> Self {
    AddrResister {
      value: (0, 0), // high byte first,lobyte second
      hi_ptr: true,
    }
  }

  pub fn set(&mut self, data: u16) {
    self.value.0 = (data >> 8) as u8;
    self.value.1 = (data & 0xFF) as u8;
  }

  pub fn update(&mut self, data: u8) {
    if self.hi_ptr {
      self.value.0 = data;
    } else {
      self.value.1 = data;
    }

    if self.get() > 0x3fff {
      self.set(self.get() & 0b11_1111_1111_1111);
    }
    self.hi_ptr = !self.hi_ptr;
  }

  pub fn increment(&mut self, inc: u8) {
    let lo = self.value.1;
    self.value.1 = self.value.1.wrapping_add(inc);
    if lo > self.value.1 {
      self.value.0 = self.value.0.wrapping_add(1);
    }
    if self.get() > 0x3FFF {
      self.set(self.get() & 0b11_1111_1111_1111);
    }
  }

  pub fn reset_latch(&mut self) {
    self.hi_ptr = true;
  }

  pub fn get(&self) -> u16 {
    ((self.value.0 as u16) << 8) | (self.value.1 as u16)
  }
}

bitflags! {
  pub struct ControlRegister: u8 {
    const NAMETABLE1              =0b0000_0001;
    const NAMETABLE2              =0b0000_0010;
    const VRAM_ADD_INCREMENT      =0b0000_0100;
    const SPRITE_PATTERN_ADDR     =0b0000_1000;
    const BACKGROUND_PATTERN_ADDR =0b0001_0000;
    const SPRITE_SIZE             =0b0010_0000;
    const MASTER_SLAVE_SELECT     =0b0100_0000;
    const GENERATE_NMI            =0b1000_0000;
  }
}
impl ControlRegister {
  pub fn new() -> Self {
    ControlRegister::from_bits_truncate(0b0000_0000)
  }

  pub fn vram_addr_increment(&self) -> u8 {
    if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
      1
    } else {
      32
    }
  }

  pub fn update(&mut self, data: u8) {
    // todo
    *self.0.bits_mut() = data;
  }

  pub fn generate_vblank_nmi(&mut self) -> bool {
    let last_status = self.contains(ControlRegister::GENERATE_NMI);
    self.set(ControlRegister::GENERATE_NMI, true);
    last_status
  }

  pub fn background_paattern_addr(&self) -> u16 {
    if !self.contains(ControlRegister::BACKGROUND_PATTERN_ADDR) {
      0x0000
    } else {
      0x1000
    }
  }

  pub fn is_sprite_8x16_mode(&self) -> bool {
    self.contains(ControlRegister::SPRITE_SIZE)
  }

  pub fn sprite_pattern_addr(&self) -> u16 {
    // ignore in 8x16 mode
    // FIXME
    // if self.is_sprite_8x16_mode() {
    //   return 0x0000;
    // }
    if !self.contains(ControlRegister::SPRITE_PATTERN_ADDR) {
      0x0000
    } else {
      0x1000
    }
  }

  pub fn nametable_addr(&self) -> u16 {
    match (
      self.contains(ControlRegister::NAMETABLE1),
      self.contains(ControlRegister::NAMETABLE2),
    ) {
      (false, false) => 0x2000,
      (false, true) => 0x2400,
      (true, false) => 0x2800,
      (true, true) => 0x2C00,
    }
  }
}

bitflags! {
  pub struct StatusRegister:u8{
    const PPU_OPEN_BUS1       =0b0000_0001;
    const PPU_OPEN_BUS2       =0b0000_0010;
    const PPU_OPEN_BUS3       =0b0000_0100;
    const PPU_OPEN_BUS4       =0b0000_1000;
    const PPU_OPEN_BUS5       =0b0001_0000;
    const SPRITE_OVERFLOW     =0b0010_0000;
    const SPRITE_ZERO_HIT     =0b0100_0000;
    const VBLANK_HAS_STARTED  =0b1000_0000;
  }
}

impl StatusRegister {
  pub fn new() -> Self {
    StatusRegister::from_bits_truncate(0b0000_0000)
  }
  pub fn is_in_vblank(&self) -> bool {
    self.contains(StatusRegister::VBLANK_HAS_STARTED)
  }
  pub fn set_vblank_status(&mut self, value: bool) {
    self.set(StatusRegister::VBLANK_HAS_STARTED, value);
  }
  pub fn reset_vblank_status(&mut self) {
    self.set_vblank_status(false);
  }
  pub fn update(&mut self, data: u8) {
    *self.0.bits_mut() = data;
  }
  pub fn set_sprite_zero_hit(&mut self, value: bool) {
    self.set(StatusRegister::SPRITE_ZERO_HIT, value)
  }
}

bitflags! {
  pub struct MaskRegister:u8{
    const GREYSCALE               = 0b0000_0001;
    const SHOW_BACKGROUND_IN_LEFT = 0b0000_0010;
    const SHOW_SPRITES_IN_LEFT    = 0b0000_0100;
    const SHOW_BACKGROUND         = 0b0000_1000;
    const SHOW_SPRITES            = 0b0001_0000;
    const EMPHASIZE_RED           = 0b0010_0000;
    const EMPHASIZE_GREEN         = 0b0100_0000;
    const EMPHASIZE_BLUE          = 0b1000_0000;
  }
}
impl MaskRegister {
  pub fn new() -> Self {
    MaskRegister::from_bits_truncate(0b0000_0000)
  }

  pub fn update(&mut self, data: u8) {
    *self.0.bits_mut() = data;
  }

  pub fn show_sprites(&self) -> bool {
    self.contains(MaskRegister::SHOW_SPRITES)
  }
}

pub struct ScrollRegister {
  pub scroll_x: u8,
  pub scroll_y: u8,
  writte_x: bool,
}

impl ScrollRegister {
  pub fn new() -> Self {
    ScrollRegister {
      scroll_x: 0,
      scroll_y: 0,
      writte_x: true,
    }
  }

  fn set(&mut self, data: u8) {
    if self.writte_x {
      self.scroll_x = data;
    } else {
      self.scroll_y = data;
    }
    self.writte_x = !self.writte_x;
  }

  pub fn reset(&mut self) {
    self.writte_x = true;
  }
}

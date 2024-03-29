use crate::frame::{self, Frame};
use crate::ppu::NesPPU;
use crate::rom::Mirroring;
use crate::{main, palette};
use log::{debug, info};

struct Rect {
  x1: usize,
  y1: usize,
  x2: usize,
  y2: usize,
}
impl Rect {
  fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
    Rect {
      x1: x1,
      y1: y1,
      x2: x2,
      y2: y2,
    }
  }
}

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
  let scroll_x = (ppu.scroll.scroll_x) as usize;
  let scroll_y = (ppu.scroll.scroll_y) as usize;

  let (main_name_table, second_name_table) = match (&ppu.mirroring, ppu.ctrl.nametable_addr()) {
    (Mirroring::VERTICAL, 0x2000) | (Mirroring::VERTICAL, 0x2800) => {
      (&ppu.vram[0x0000..0x0400], &ppu.vram[0x0400..0x0800])
    }
    (Mirroring::VERTICAL, 0x2400) | (Mirroring::VERTICAL, 0x2C00) => {
      (&ppu.vram[0x0400..0x0800], &ppu.vram[0x0000..0x0400])
    }
    // FIXME 間違えてる
    (Mirroring::HORIZONTAL, 0x2000) | (Mirroring::HORIZONTAL, 0x2400) => {
      (&ppu.vram[0x000..0x400], &ppu.vram[0x400..0x800])
    }
    (Mirroring::HORIZONTAL, 0x2800) | (Mirroring::HORIZONTAL, 0x2C00) => {
      (&ppu.vram[0x400..0x800], &ppu.vram[0x000..0x400])
    }
    (_, _) => {
      panic!("Not supported mirroring type {:?}", ppu.mirroring);
    }
  };

  let screen_w = 256;
  let screen_h = 240;

  // 左上
  render_name_table(
    ppu,
    frame,
    main_name_table,
    Rect::new(scroll_x, scroll_y, screen_w, screen_h),
    -(scroll_x as isize),
    -(scroll_y as isize),
  );

  // 右下
  render_name_table(
    ppu,
    frame,
    second_name_table,
    Rect::new(0, 0, scroll_x, 240),
    (screen_w - scroll_x) as isize,
    (screen_h - scroll_y) as isize,
  );

  // 左下
  render_name_table(
    ppu,
    frame,
    main_name_table,
    Rect::new(scroll_x, scroll_y, screen_w, screen_h),
    -(scroll_x as isize),
    (screen_h - scroll_y) as isize,
  );

  // 右上
  render_name_table(
    ppu,
    frame,
    second_name_table,
    Rect::new(0, 0, scroll_x, 240),
    (screen_w - scroll_x) as isize,
    -(scroll_y as isize),
  );

  // ===========================================================================================
  // Draw Sprites
  // ===========================================================================================
  for i in (0..ppu.oam_data.len()).step_by(4).rev() {
    let tile_y = ppu.oam_data[i] as usize;
    let tile_idx = ppu.oam_data[i + 1] as u16;
    let attr = ppu.oam_data[i + 2] as u8;
    let tile_x = ppu.oam_data[i + 3] as usize;

    let flip_vertical = (attr >> 7 & 1) == 1;
    let flip_horizontal = (attr >> 6 & 1) == 1;
    let pallette_idx = attr & 0b11;
    let sprite_pallette = sprite_palette(ppu, tile_y, pallette_idx);

    let bank: u16 = ppu.ctrl.sprite_pattern_addr();
    let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];

    for y in 0..=7 {
      let mut upper = tile[y];
      let mut lower = tile[y + 8];
      'ololo: for x in (0..=7).rev() {
        let value = (1 & lower) << 1 | (1 & upper);
        upper = upper >> 1;
        lower = lower >> 1;
        let rgb = match value {
          0 => continue 'ololo, // skip coloring the pixel
          1 => palette::SYSTEM_PALLETE[sprite_pallette[1] as usize],
          2 => palette::SYSTEM_PALLETE[sprite_pallette[2] as usize],
          3 => palette::SYSTEM_PALLETE[sprite_pallette[3] as usize],
          _ => panic!("can't be"),
        };
        match (flip_horizontal, flip_vertical) {
          (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
          (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
          (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
          (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
        }
      }
    }
  }
}

fn bg_pallete(ppu: &NesPPU, attribute_table: &[u8], tile_colum: usize, tile_row: usize) -> [u8; 4] {
  let attr_table_idx = tile_row / 4 * 8 + tile_colum / 4;
  let attr_byte = attribute_table[attr_table_idx];

  let pallet_idx = match (tile_colum % 4 / 2, tile_row % 4 / 2) {
    (0, 0) => attr_byte & 0b11,
    (1, 0) => (attr_byte >> 2) & 0b11,
    (0, 1) => (attr_byte >> 4) & 0b11,
    (1, 1) => (attr_byte >> 6) & 0b11,
    (_, _) => panic!("should not happen"),
  };

  let pallete_start: usize = 1 + (pallet_idx as usize) * 4;
  let p = ppu.read_palette_table(tile_row * 8);
  [
    p[0],
    p[pallete_start],
    p[pallete_start + 1],
    p[pallete_start + 2],
  ]
}

fn sprite_palette(ppu: &NesPPU, tile_y: usize, pallete_idx: u8) -> [u8; 4] {
  let start = 0x11 + (pallete_idx * 4) as usize;
  let p = ppu.read_palette_table(tile_y);
  [0, p[start], p[start + 1], p[start + 2]]
}

fn render_name_table(
  ppu: &NesPPU,
  frame: &mut Frame,
  name_table: &[u8],
  view_port: Rect,
  shift_x: isize,
  shift_y: isize,
) {
  let bank = ppu.ctrl.background_pattern_addr();
  let attribute_table = &name_table[0x03C0..0x0400];

  for i in 0..0x03C0 {
    let tile_colum = i % 32;
    let tile_row = i / 32;
    let tile_idx = name_table[i] as u16;
    let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];
    let palette = bg_pallete(ppu, attribute_table, tile_colum, tile_row);

    for y in 0..=7 {
      let mut upper = tile[y];
      let mut lower = tile[y + 8];

      for x in (0..=7).rev() {
        // タイルを組み合わせている
        let value = (1 & lower) << 1 | (1 & upper);
        // タイルを横に
        upper = upper >> 1;
        lower = lower >> 1;
        let rgb = match value {
          0 => palette::SYSTEM_PALLETE[palette[0] as usize],
          1 => palette::SYSTEM_PALLETE[palette[1] as usize],
          2 => palette::SYSTEM_PALLETE[palette[2] as usize],
          3 => palette::SYSTEM_PALLETE[palette[3] as usize],
          _ => panic!("can't be"),
        };

        let pixel_x = tile_colum * 8 + x;
        let pixel_y = tile_row * 8 + y;
        if pixel_x >= view_port.x1
          && pixel_x < view_port.x2
          && pixel_y >= view_port.y1
          && pixel_y < view_port.y2
        {
          frame.set_pixel(
            (shift_x + pixel_x as isize) as usize,
            (shift_y + pixel_y as isize) as usize,
            rgb,
          )
        }
      }
    }
  }
}

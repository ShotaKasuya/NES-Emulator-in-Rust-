use crate::frame::{self, Frame};
use crate::palette;
use crate::ppu::NesPPU;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
  // ===========================================================================================
  // Draw BackGround
  // ===========================================================================================
  let bank = ppu.ctrl.bknd_paattern_addr();
  for i in 0..0x03C0 {
    let tile = ppu.vram[i] as u16;
    let tile_x = i % 32;
    let tile_y = i / 32;
    let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
    let palette = bg_pallete(ppu, tile_x, tile_y);

    for y in 0..=7 {
      let mut upper = tile[y];
      let mut lower = tile[y + 8];

      for x in (0..=7).rev() {
        let value = (1 & upper) << 1 | (1 & lower);
        upper = upper >> 1;
        lower = lower >> 1;
        let rgb = match value {
          // 0 => palette::SYSTEM_PALLETE[0x01],
          // 1 => palette::SYSTEM_PALLETE[0x23],
          // 2 => palette::SYSTEM_PALLETE[0x27],
          // 3 => palette::SYSTEM_PALLETE[0x30],
          0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
          1 => palette::SYSTEM_PALLETE[palette[1] as usize],
          2 => palette::SYSTEM_PALLETE[palette[2] as usize],
          3 => palette::SYSTEM_PALLETE[palette[3] as usize],
          _ => panic!("can't be"),
        };
        frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
      }
    }
  }

  // ===========================================================================================
  // Draw Sprites
  // ===========================================================================================
  for i in (0..ppu.oam_data.len()).step_by(4).rev() {
    let tile_idx = ppu.oam_data[i + 1] as u16;
    let tile_x = ppu.oam_data[i + 3] as usize;
    let tile_y = ppu.oam_data[i] as usize;
    let addr = ppu.oam_data[i + 2] as u8;

    let flip_vertical = addr >> 7 & 1 == 1;
    let flip_horizontal = addr >> 6 & 1 == 1;
    let pallette_idx = addr & 0b11;
    let sprite_pallette = sprite_palette(ppu, pallette_idx);

    let bank: u16 = ppu.ctrl.sprt_pattern_addr();
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
          (true, false) => frame.set_pixel(tile_x + 8 - x, tile_y + y, rgb),
          (false, true) => frame.set_pixel(tile_x + x, tile_y + 8 - y, rgb),
          (true, true) => frame.set_pixel(tile_x + 8 + x, tile_y + 8 + y, rgb),
        }
      }
    }
  }
}

fn bg_pallete(ppu: &NesPPU, tile_colum: usize, tile_row: usize) -> [u8; 4] {
  let attr_table_idx = tile_row / 4 * 8 + tile_colum / 4;
  let attr_byte = ppu.vram[0x3C0 + attr_table_idx];

  let pallet_idx = match (tile_colum % 4 / 2, tile_row % 4 / 2) {
    (0, 0) => attr_byte & 0b11,
    (1, 0) => (attr_byte >> 2) & 0b11,
    (0, 1) => (attr_byte >> 4) & 0b11,
    (1, 1) => (attr_byte >> 6) & 0b11,
    (_, _) => panic!("should not happen"),
  };

  let pallete_start: usize = 1 + (pallet_idx as usize) * 4;
  [
    ppu.palette_table[0],
    ppu.palette_table[pallete_start],
    ppu.palette_table[pallete_start + 1],
    ppu.palette_table[pallete_start + 2],
  ]
}

fn sprite_palette(ppu: &NesPPU, pallete_idx: u8) -> [u8; 4] {
  let start = 0x11 + (pallete_idx * 4) as usize;
  [
    0,
    ppu.palette_table[start],
    ppu.palette_table[start + 1],
    ppu.palette_table[start + 2],
  ]
}

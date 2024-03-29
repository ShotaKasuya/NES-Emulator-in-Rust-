use std::{fs::File, io::Read};

use crate::rom::Rom;

pub fn load_rom(path: &str) -> Rom {
  let mut f = File::open(path).expect("no file found");
  let metadata = std::fs::metadata(path).expect("unable to read metadata");
  let mut buffer = vec![0; metadata.len() as usize];
  f.read(&mut buffer).expect("buffer overflow");
  let rom = Rom::new(&buffer).expect("load errpr");
  rom
}

#[allow(dead_code)]
pub fn snake_rom() -> Rom {
  load_rom("rom/snake.nes")
}
#[allow(dead_code)]
pub fn test_rom() -> Rom {
  load_rom("rom/nestest.nes")
}
#[allow(dead_code)]
pub fn alter_ego_rom() -> Rom {
  load_rom("rom/Alter_Ego.nes")
}
#[allow(dead_code)]
pub fn bomb_sweeper_rom() -> Rom {
  load_rom("rom/BombSweeper.nes")
}

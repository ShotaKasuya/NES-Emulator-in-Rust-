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

pub mod test {
  use super::*;

  pub fn snake_rom() -> Rom {
    load_rom("rom/snake.nes")
  }
  pub fn test_rom() -> Rom {
    load_rom("rom/nestest.nes")
  }
}

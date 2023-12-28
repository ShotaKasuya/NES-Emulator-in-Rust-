use std::{fs::File, io::Read};

use crate::rom::Rom;

pub mod test {
  use super::*;
  pub fn test_rom() -> Rom {
    let mut f = File::open("rom/nestest.nes").expect("no file found");
    let metadata = std::fs::metadata("rom/nestest.nes").expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");
    let rom = Rom::new(&buffer).expect("load errpr");
    rom
  }
}

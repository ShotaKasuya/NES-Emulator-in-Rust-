use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub struct NesAPU {
  ch1_register: Ch1Register,
  ch2_register: Ch2Register,
  ch3_register: Ch3Register,
  ch4_register: Ch4Register,

  ch1_device: AudioDevice<SquareWave>,
  ch1_sender: Sender<SquareNote>,

  ch2_device: AudioDevice<SquareWave>,
  ch2_sender: Sender<SquareNote>,

  ch3_device: AudioDevice<TriangleWave>,
  ch3_sender: Sender<TriangleNote>,

  ch4_device: AudioDevice<NoiseWave>,
  ch4_sender: Sender<NoiseNote>,
}

const NES_CPU_CLOCK: f32 = 1_789_773.0; //1.78Hz
impl NesAPU {
  pub fn new(sdl_context: &sdl2::Sdl) -> Self {
    let (ch1_device, ch1_sender) = init_square(sdl_context);
    let (ch2_device, ch2_sender) = init_square(sdl_context);
    let (ch3_device, ch3_sender) = init_triangle(sdl_context);
    let (ch4_device, ch4_sender) = init_noise(sdl_context);

    NesAPU {
      ch1_register: Ch1Register::new(),
      ch3_register: Ch3Register::new(),
      ch2_register: Ch2Register::new(),
      ch4_register: Ch4Register::new(),

      ch1_device: ch1_device,
      ch1_sender: ch1_sender,
      ch2_device: ch2_device,
      ch2_sender: ch2_sender,
      ch3_device: ch3_device,
      ch3_sender: ch3_sender,
      ch4_device: ch4_device,
      ch4_sender: ch4_sender,
    }
  }

  pub fn write_1ch(&mut self, addr: u16, volume: u8) {
    self.ch1_register.write(addr, volume);

    let duty = match self.ch1_register.duty() {
      0b00 => 0.125,
      0b01 => 0.25,
      0b10 => 0.50,
      0b11 => 0.75,
      _ => panic!("can't be"),
    };

    let volume = (self.ch1_register.volume() as f32) / 15.0;
    let hz = NES_CPU_CLOCK / (16.0 * (self.ch1_register.hz() as f32 + 1.0));

    self
      .ch1_sender
      .send(SquareNote {
        hz: hz,
        volume: volume,
        duty: duty,
      })
      .unwrap();
  }

  pub fn write_2ch(&mut self, addr: u16, value: u8) {
    self.ch2_register.write(addr, value);

    let duty = match self.ch2_register.duty {
      0x00 => 0.125,
      0x01 => 0.25,
      0x02 => 0.50,
      0x03 => 0.75,
      _ => panic!("can't be",),
    };

    let volume = (self.ch2_register.volume as f32) / 15.0;

    let hz = NES_CPU_CLOCK / (16.0 * (self.ch2_register.frequency as f32 + 1.0));

    self
      .ch2_sender
      .send(SquareNote {
        hz: hz,
        volume: volume,
        duty: duty,
      })
      .unwrap();
  }

  pub fn write_3ch(&mut self, addr: u16, value: u8) {
    self.ch3_register.write(addr, value);

    let hz = NES_CPU_CLOCK / (16.0 * (self.ch3_register.frequency as f32 + 1.0));
    self.ch3_sender.send(TriangleNote { hz: hz }).unwrap();
  }

  pub fn write_4ch(&mut self, addr: u16, value: u8) {
    self.ch4_register.write(addr, value);

    let hz = NES_CPU_CLOCK / NOIZE_TABLE[self.ch4_register.frequency as usize] as f32;
    let is_long = match self.ch4_register.kind {
      NoiseKind::Long => true,
      _ => false,
    };
    let volume = (self.ch4_register.volume as f32) / 15.0;
    self
      .ch4_sender
      .send(NoiseNote {
        hz: hz,
        is_long: is_long,
        volume: volume,
      })
      .unwrap();
  }
}

struct Ch1Register {
  tone_volume: u8,
  sweep: u8,
  hz_low: u8,
  hz_high_key_on: u8,
}
impl Ch1Register {
  pub fn new() -> Self {
    Ch1Register {
      tone_volume: 0x00,
      sweep: 0x00,
      hz_low: 0x00,
      hz_high_key_on: 0x00,
    }
  }

  pub fn duty(&self) -> u8 {
    // 00:12.5%
    // 01:  25%
    // 10:  50%
    // 11:  75%
    (self.tone_volume & 0b1100_0000) >> 6
  }

  pub fn volume(&self) -> u8 {
    // 0で消音、15が最大
    self.tone_volume & 0b0000_1111
  }

  pub fn hz(&self) -> u16 {
    (self.hz_high_key_on as u16 & 0b0000_0111 << 8) | self.hz_low as u16
  }

  pub fn write(&mut self, addr: u16, volume: u8) {
    match addr {
      0x4000 => {
        self.tone_volume = volume;
      }
      0x4001 => {
        self.sweep = volume;
      }
      0x4002 => {
        self.hz_low = volume;
      }
      0x4003 => {
        self.hz_high_key_on = volume;
      }
      _ => panic!("can't be"),
    }
  }
}

struct Ch2Register {
  volume: u8,
  envelope_flag: bool,
  key_off_counter_flag: bool,
  duty: u8,

  sweep_change_amount: u8,
  sweep_direction: u8,
  sweep_timer_count: u8,
  sweep_enabled: u8,

  frequency: u16,
  key_off_count: u8,
}
impl Ch2Register {
  pub fn new() -> Self {
    Ch2Register {
      volume: 0x00,
      envelope_flag: false,
      key_off_counter_flag: false,
      duty: 0,

      sweep_change_amount: 0,
      sweep_direction: 0,
      sweep_timer_count: 0,
      sweep_enabled: 0,

      frequency: 0,
      key_off_count: 0,
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    match addr {
      0x4004 => {
        self.volume = value & 0x0F;
        self.envelope_flag = (value & 0x10) == 0;
        self.key_off_counter_flag = (value & 0x20) == 0;
        self.duty = (value & 0xC0) >> 6;
      }
      0x4005 => {
        self.sweep_change_amount = value & 0x07;
        self.sweep_direction = (value & 0x08) >> 3;
        self.sweep_timer_count = (value & 0x70) >> 4;
        self.sweep_enabled = (value & 0x80) >> 7;
      }
      0x4006 => {
        self.frequency = (self.frequency & 0x0700) | value as u16;
      }
      0x4007 => {
        self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
        self.key_off_count = (value & 0xF8) >> 3;
      }
      _ => panic!("can't be"),
    }
  }
}

struct Ch3Register {
  // 4008
  length: u8,
  key_off_counter_flag: bool,

  // 400A,400B
  frequency: u16,
  key_off_count: u8,
}

impl Ch3Register {
  pub fn new() -> Self {
    Ch3Register {
      length: 0,
      key_off_counter_flag: false,
      frequency: 0,
      key_off_count: 0,
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    match addr {
      0x4008 => {
        self.length = value & 0x7F;
        self.key_off_counter_flag = (value & 0x80) == 0;
      }
      0x4009 => {}
      0x400A => {
        self.frequency = (self.frequency & 0x0700) | value as u16;
      }
      0x400B => {
        self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
      }
      _ => panic!("can't be"),
    }
  }
}

enum NoiseKind {
  Long,
  Short,
}

struct Ch4Register {
  // 400C
  volume: u8,
  envelope_flag: bool,
  key_off_counter_flag: bool,

  // 400E
  frequency: u8,
  kind: NoiseKind,

  // 400F
  key_off_count: u8,
}

impl Ch4Register {
  pub fn new() -> Self {
    Ch4Register {
      volume: 0,
      envelope_flag: false,
      key_off_counter_flag: false,
      frequency: 0,
      kind: NoiseKind::Long,
      key_off_count: 0,
    }
  }

  pub fn write(&mut self, addr: u16, value: u8) {
    match addr {
      0x400C => {
        self.volume = value & 0x0F;
        self.envelope_flag = (value & 0x10) == 0;
        self.key_off_counter_flag = (value & 0x20) == 0;
      }
      0x400E => {
        self.frequency = value & 0x0F;
        self.kind = match value & 0x80 {
          0 => NoiseKind::Long,
          _ => NoiseKind::Short,
        };
      }
      0x400F => {
        self.key_off_count = (value & 0xF8) >> 3;
      }
      _ => panic!("can't be"),
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
struct SquareNote {
  hz: f32,
  volume: f32,
  duty: f32,
}

struct SquareWave {
  freq: f32,
  phase: f32,
  receiver: Receiver<SquareNote>,
  note: SquareNote,
}

impl AudioCallback for SquareWave {
  type Channel = f32;

  fn callback(&mut self, out: &mut [f32]) {
    for x in out.iter_mut() {
      let res = self.receiver.recv_timeout(Duration::from_millis(0));
      match res {
        Ok(note) => self.note = note,
        Err(_) => {}
      }
      *x = if self.phase <= self.note.duty {
        self.note.volume
      } else {
        -self.note.volume
      };
      self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
    }
  }
}

fn init_square(sdl_context: &sdl2::Sdl) -> (AudioDevice<SquareWave>, Sender<SquareNote>) {
  let audio_subsystem = sdl_context.audio().unwrap();

  let (sender, receiver) = channel::<SquareNote>();

  let desired_spec = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: None,
  };

  let device = audio_subsystem
    .open_playback(None, &desired_spec, |spec| SquareWave {
      freq: spec.freq as f32,
      phase: 0.0,
      receiver: receiver,
      note: SquareNote {
        hz: 0.0,
        volume: 0.0,
        duty: 0.0,
      },
    })
    .unwrap();

  device.resume();

  (device, sender)
}

lazy_static! {
  pub static ref NOIZE_TABLE: Vec<u16> = vec![
    0x0002, 0x0004, 0x0008, 0x0010, 0x0020, 0x0030, 0x0040, 0x0050, 0x0065, 0x007F, 0x00BE, 0x00FE,
    0x017D, 0x01FC, 0x03F9, 0x07F2,
  ];
}

#[derive(Debug, Clone, PartialEq)]
struct NoiseNote {
  hz: f32,
  is_long: bool,
  volume: f32,
}

struct NoiseWave {
  freq: f32,
  phase: f32,
  receiver: Receiver<NoiseNote>,
  value: bool,
  long_random: NoiseRandom,
  short_random: NoiseRandom,

  note: NoiseNote,
}

impl AudioCallback for NoiseWave {
  type Channel = f32;

  fn callback(&mut self, out: &mut [Self::Channel]) {
    for x in out.iter_mut() {
      let res = self.receiver.recv_timeout(Duration::from_millis(0));
      match res {
        Ok(note) => self.note = note,
        Err(_) => {}
      }

      *x = if self.value { 0.0 } else { 1.0 } * self.note.volume;

      let last_phase = self.phase;
      self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
      if last_phase > self.phase {
        self.value = if self.note.is_long {
          self.long_random.next()
        } else {
          self.short_random.next()
        }
      }
    }
  }
}

struct NoiseRandom {
  bit: u8,
  value: u16,
}
impl NoiseRandom {
  pub fn long() -> Self {
    NoiseRandom { bit: 1, value: 1 }
  }
  pub fn short() -> Self {
    NoiseRandom { bit: 6, value: 1 }
  }

  pub fn next(&mut self) -> bool {
    let b = (self.value & 0x01) ^ ((self.value >> self.bit) & 0x01);
    self.value = self.value >> 1;
    self.value = self.value & 0b0011_1111_1111_1111 | b << 14;

    self.value & 0x01 != 0
  }
}

fn init_noise(sdl_context: &sdl2::Sdl) -> (AudioDevice<NoiseWave>, Sender<NoiseNote>) {
  let audio_subsystem = sdl_context.audio().unwrap();

  let (sender, receiver) = channel::<NoiseNote>();

  let desired_spec = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: None,
  };

  let device = audio_subsystem
    .open_playback(None, &desired_spec, |spec| NoiseWave {
      freq: spec.freq as f32,
      phase: 0.0,
      receiver: receiver,
      value: false,
      long_random: NoiseRandom::long(),
      short_random: NoiseRandom::short(),
      note: NoiseNote {
        hz: 0.0,
        is_long: true,
        volume: 0.0,
      },
    })
    .unwrap();

  device.resume();
  (device, sender)
}

#[derive(Debug, Clone, PartialEq)]
struct TriangleNote {
  hz: f32,
}

struct TriangleWave {
  freq: f32,
  phase: f32,
  receiver: Receiver<TriangleNote>,
  note: TriangleNote,
}
impl AudioCallback for TriangleWave {
  type Channel = f32;

  fn callback(&mut self, out: &mut [Self::Channel]) {
    for x in out.iter_mut() {
      let res = self.receiver.recv_timeout(Duration::from_millis(0));
      match res {
        Ok(note) => self.note = note,
        Err(_) => {}
      }
      *x = (if self.phase <= 0.5 {
        self.phase
      } else {
        1.0 - self.phase
      } - 0.25)
        * 2.0;

      self.phase = (self.phase + self.note.hz / self.freq) % 1.0;
    }
  }
}

fn init_triangle(sdl_context: &sdl2::Sdl) -> (AudioDevice<TriangleWave>, Sender<TriangleNote>) {
  let audio_subsystem = sdl_context.audio().unwrap();

  let (sender, receiver) = channel::<TriangleNote>();
  let desired_spec = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: None,
  };
  let device = audio_subsystem
    .open_playback(None, &desired_spec, |spec| TriangleWave {
      freq: spec.freq as f32,
      phase: 0.0,
      receiver: receiver,
      note: TriangleNote { hz: 0.0 },
    })
    .unwrap();

  device.resume();
  (device, sender)
}

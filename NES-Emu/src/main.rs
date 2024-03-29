extern crate sdl2;

use std::collections::HashMap;

use crate::bus::Mem;
use crate::cpu::{trace, CPU};
use apu::NesAPU;
use bus::Bus;
use cartridge::bomb_sweeper_rom;
use cartridge::{alter_ego_rom, test_rom};
use frame::show_tile;
use frame::Frame;
use joypad::Joypad;
use log::trace;
use ppu::NesPPU;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

#[macro_use]
extern crate lazy_static;

mod apu;
mod bus;
mod cartridge;
mod cpu;
mod frame;
mod joypad;
mod opscodes;
mod palette;
mod ppu;
mod render;
mod rom;

fn main() {
  env_logger::init();

  // init sdl2
  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  let window = video_subsystem
    .window("Nes Emurator", (256.0 * 2.0) as u32, (240.0 * 2.0) as u32)
    .position_centered()
    .build()
    .unwrap();

  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  let mut event_pump = sdl_context.event_pump().unwrap();
  canvas.set_scale(2.0, 2.0).unwrap();

  let creator = canvas.texture_creator();
  let mut texture = creator
    .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
    .unwrap();

  // put CHR_ROM
  // let rom = bomb_sweeper_rom();
  let rom = alter_ego_rom();
  let apu = NesAPU::new(&sdl_context);
  let mut frame = Frame::new();

  let mut key_map = HashMap::new();
  key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
  key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
  key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
  key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
  key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
  key_map.insert(Keycode::Return, joypad::JoypadButton::START);
  key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
  key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

  let bus = Bus::new(rom, apu, move |ppu: &NesPPU, joypad1: &mut Joypad| {
    // println!("***GAME LOOP***");
    render::render(ppu, &mut frame);
    texture.update(None, &frame.data, 256 * 3).unwrap();

    canvas.copy(&texture, None, None).unwrap();

    canvas.present();
    for event in event_pump.poll_iter() {
      match event {
        Event::Quit { .. }
        | Event::KeyDown {
          keycode: Some(Keycode::Escape),
          ..
        } => std::process::exit(0),

        Event::KeyDown { keycode, .. } => {
          if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
            joypad1.set_button_pressed_status(*key, true);
          }
        }
        Event::KeyUp { keycode, .. } => {
          if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
            joypad1.set_button_pressed_status(*key, false);
          }
        }
        _ => { /* do nothing */ }
      }
    }
  });

  let mut cpu = CPU::new(bus);

  cpu.reset();
  cpu.run_with_callback(move |cpu| {
    trace!("{}", trace(cpu));
  });
  /*
  let mut screen_state = [0 as u8; 32 * 3 * 32];
  let mut rng = rand::thread_rng();
  cpu.run_with_callback(move |cpu| {
    println!("{}", trace(cpu));

    handle_user_input(cpu, &mut event_pump);
    cpu.mem_write(0xfe, rng.gen_range(1..16));

    if read_screen_state(cpu, &mut screen_state) {
      texture.update(None, &screen_state, 32 * 3).unwrap();
      canvas.copy(&texture, None, None).unwrap();
      canvas.present();
    }

    ::std::thread::sleep(std::time::Duration::new(0, 70_000));
  })
  */
}

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
  for event in event_pump.poll_iter() {
    match event {
      Event::Quit { .. }
      | Event::KeyDown {
        keycode: Some(Keycode::Escape),
        ..
      } => std::process::exit(1),
      Event::KeyDown {
        keycode: Some(Keycode::W),
        ..
      } => {
        cpu.mem_write(0xFF, 0x77);
      }
      Event::KeyDown {
        keycode: Some(Keycode::S),
        ..
      } => {
        cpu.mem_write(0xFF, 0x73);
      }
      Event::KeyDown {
        keycode: Some(Keycode::A),
        ..
      } => {
        cpu.mem_write(0xFF, 0x61);
      }
      Event::KeyDown {
        keycode: Some(Keycode::D),
        ..
      } => {
        cpu.mem_write(0xFF, 0x64);
      }
      _ => { /* do nothing */ }
    }
  }
}

fn color(byte: u8) -> Color {
  match byte {
    0 => sdl2::pixels::Color::BLACK,
    1 => sdl2::pixels::Color::WHITE,
    2 | 9 => sdl2::pixels::Color::GRAY,
    3 | 10 => sdl2::pixels::Color::RED,
    4 | 11 => sdl2::pixels::Color::GREEN,
    5 | 12 => sdl2::pixels::Color::BLUE,
    6 | 13 => sdl2::pixels::Color::MAGENTA,
    7 | 14 => sdl2::pixels::Color::YELLOW,
    _ => sdl2::pixels::Color::CYAN,
  }
}

fn read_screen_state(cpu: &mut CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
  let mut frame_idx = 0;
  let mut update = false;
  for i in 0x0200..0x600 {
    let color_idx = cpu.mem_read(i as u16);
    let (b1, b2, b3) = color(color_idx).rgb();
    if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
      frame[frame_idx] = b1;
      frame[frame_idx + 1] = b2;
      frame[frame_idx + 2] = b3;
      update = true;
    }
    frame_idx += 3;
  }
  update
}

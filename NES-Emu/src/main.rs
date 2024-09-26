use pixels::{Pixels, SurfaceTexture};
use std::collections::HashMap;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::WindowBuilder;

use nes_emu::bus::Bus;
use nes_emu::cartridge::bomb_sweeper_rom;
use nes_emu::cpu::trace;
use nes_emu::cpu::CPU;
use nes_emu::frame::Frame;
use nes_emu::joypad::*;
use nes_emu::ppu::NesPPU;
use nes_emu::render;

const WINDOW_PIXEL_WIDTH: u32 = 256;
const WINDOW_PIXEL_HEIGHT: u32 = 240;

fn main() {
  env_logger::init();

  let mut event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_title("nes_emulator")
    .with_inner_size(LogicalSize::new(
      WINDOW_PIXEL_WIDTH * 2,
      WINDOW_PIXEL_HEIGHT * 2,
    ))
    .build(&event_loop)
    .unwrap();
  let mut pixels = {
    let window_size = window.inner_size();
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    Pixels::new(WINDOW_PIXEL_WIDTH, WINDOW_PIXEL_HEIGHT, surface_texture).unwrap()
  };

  // put CHR_ROM
  // let rom = bomb_sweeper_rom();
  let rom = bomb_sweeper_rom();
  // let apu = NesAPU::new(&sdl_context);
  let mut frame = Frame::new();

  let mut key_map = HashMap::new();
  key_map.insert(VirtualKeyCode::Down, JoypadButton::DOWN);
  key_map.insert(VirtualKeyCode::Up, JoypadButton::UP);
  key_map.insert(VirtualKeyCode::Right, JoypadButton::RIGHT);
  key_map.insert(VirtualKeyCode::Left, JoypadButton::LEFT);
  key_map.insert(VirtualKeyCode::Space, JoypadButton::SELECT);
  key_map.insert(VirtualKeyCode::Return, JoypadButton::START);
  key_map.insert(VirtualKeyCode::A, JoypadButton::BUTTON_A);
  key_map.insert(VirtualKeyCode::S, JoypadButton::BUTTON_B);

  let bus = Bus::new(rom, move |ppu: &NesPPU, joypad1: &mut Joypad| {
    // println!("***GAME LOOP***");

    render::render(ppu, &mut frame);

    let pixels_frame = pixels.frame_mut();
    let mut index = 0;
    for pixel in pixels_frame.chunks_exact_mut(4) {
      pixel[0] = frame.data[index];
      pixel[1] = frame.data[index + 1];
      pixel[2] = frame.data[index + 2];
      index += 3;
    }

    pixels.render().unwrap();
    event_loop.run_return(|event, _, control_flow_inner| {
      match event {
        Event::WindowEvent {
          event: WindowEvent::KeyboardInput { input, .. },
          ..
        } => {
          if let Some(keycode) = input.virtual_keycode {
            match input.state {
              ElementState::Pressed => {
                if keycode == VirtualKeyCode::Escape {
                  std::process::exit(0);
                }
                if let Some(key) = key_map.get(&keycode) {
                  joypad1.set_button_pressed_status(*key, true);
                }
              }
              ElementState::Released => {
                if let Some(key) = key_map.get(&keycode) {
                  joypad1.set_button_pressed_status(*key, false);
                }
              }
            }
          }
        }
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          std::process::exit(0);
        }
        _ => {}
      }
      if event == Event::MainEventsCleared {
        *control_flow_inner = ControlFlow::Exit;
      } else {
        *control_flow_inner = ControlFlow::Poll;
      }
    });
  });

  let mut cpu = CPU::new(bus);

  cpu.reset();
  cpu.run_with_callback(move |cpu| {
    println!("{:?}", trace(cpu));
  });
}

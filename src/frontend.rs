use crate::chip8::Chip8;

use std::thread;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Instant,Duration};

const SCALE:u32 = 20;
const INSTRUCTIONS_PER_SEC:u16 = 600;
pub fn run() {
    let mut chip8 = Chip8::default();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Rust CHIP-8 Emulator", 64*SCALE, 32*SCALE)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24,64,32).unwrap();

    let mut color_buffer = [0_u8; 3 * 64 * 32];
    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut frame_count = 0;
    'running: loop {
        let start = Instant::now();
        let input_result = handle_input(&mut event_pump,&mut chip8);
        if input_result.0 {
            break 'running
        }

        if chip8.file_loaded {chip8.step()};
        if frame_count == 0 {
            chip8.decrement_timers();
        }
        frame_count = (frame_count + 1) % 10;

        //redraw if game is running, or if it's the first frame after unload
        if chip8.screen_updated || input_result.1 {
            for i in 0..64*32 {
                let rgb_triplet = if chip8.screen_state[i] {
                    (255,100,100)
                } else {
                    (0,0,0)
                };
                (color_buffer[i*3],color_buffer[i*3+1],color_buffer[i*3+2]) = rgb_triplet;
            }
            texture.update(None, &color_buffer, 3 * 64).expect("Failed texture update");
        }
        canvas.copy(&texture,None,None).unwrap();
        canvas.present();
        let elapsed = start.elapsed();
        let wait = Duration::new(0,1_666_667).checked_sub(elapsed);
        if let Some(dur) = wait {
            thread::sleep(dur);
        }
    }
}
fn get_button(key: Keycode) -> Option<u16> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q    => Some(0x4),
        Keycode::W    => Some(0x5),
        Keycode::E    => Some(0x6),
        Keycode::R    => Some(0xD),
        Keycode::A    => Some(0x7),
        Keycode::S    => Some(0x8),
        Keycode::D    => Some(0x9),
        Keycode::F    => Some(0xE),
        Keycode::Z    => Some(0xA),
        Keycode::X    => Some(0x0),
        Keycode::C    => Some(0xB),
        Keycode::V    => Some(0xF),
        _ => None,
    }
}
//first bool if break, second bool if chip8 has been unloaded
fn handle_input(event_pump: &mut sdl2::EventPump,chip8: &mut Chip8) -> (bool,bool) {
    /* write to the previous input data field */
    for (ind,held) in chip8.input_data.iter().enumerate() {
        chip8.prev_input_data[ind] = *held;
    }
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit {..} |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return (true,false);
            },
            /* file loading keys */
            Event::KeyDown {keycode: Some(Keycode::Y), ..} => {
                chip8.read_file("roms/1-chip8-logo.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::U), ..} => {
                chip8.read_file("roms/2-ibm-logo.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::I), ..} => {
                chip8.read_file("roms/3-corax+.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::O), ..} => {
                chip8.read_file("roms/4-flags.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::P), ..} => {
                chip8.read_file("roms/5-quirks.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::LeftBracket), ..} => {
                chip8.read_file("roms/6-keypad.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::RightBracket), ..} => {
                chip8.read_file("roms/Space_Invaders.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::J), ..} => {
                chip8.read_file("roms/Breakout.ch8");
            }
            Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                *chip8 = Chip8::default();
                return (false,true);
            }
            /* input keys */
            Event::KeyDown {keycode: Some(key), ..} => {
                if let Some(ind) = get_button(key) {
                    chip8.input_data[ind as usize] = true;
                }
            }
            Event::KeyUp {keycode: Some(key), ..} => {
                if let Some(ind) = get_button(key) {
                    chip8.input_data[ind as usize] = false;
                }
            }
            _ => {}
        }
    }
    (false,false)
}

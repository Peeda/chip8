use crate::chip8::Chip8;

use std::thread;

use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const SCALE:u32 = 20;
const INSTRUCTIONS_PER_SEC:u16 = 600;
pub fn run() {
    let mut chip8 = Chip8::default();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Rust CHIP-8 Emulator", 64*SCALE, 32*SCALE)
        .opengl()
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let creator = canvas.texture_creator();
    let mut texture = creator.create_texture_target(PixelFormatEnum::RGB24,64,32).unwrap();

    let mut color_buffer = [0_u8; 3 * 64 * 32];
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        
        /* write to the previous input data field */
        for (ind,held) in chip8.input_data.iter().enumerate() {
            chip8.prev_input_data[ind] = *held;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
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
                Event::KeyDown {keycode: Some(Keycode::Space), ..} => {
                    chip8 = Chip8::default();
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

        if chip8.file_loaded {chip8.step()};

        for color in color_buffer.iter_mut().take(3*64*32) {
            *color = 0;
        }

        for i in 0..64*32 {
            if chip8.screen_state[i] {
                color_buffer[i*3] = 255;
                color_buffer[i*3+1] = 100;
                color_buffer[i*3+2] = 100;
            }
        }

        canvas.clear();

        texture.update(None, &color_buffer, 3 * 64).expect("Failed texture update");
        canvas.copy(&texture,None,None).unwrap();
        canvas.present();
        // thread::sleep(Duration::from_millis(50));
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
}

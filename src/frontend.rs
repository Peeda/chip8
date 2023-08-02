use crate::chip8::Chip8;

use std::{thread,time};

use sdl2::pixels::{Color,PixelFormatEnum};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

const SCALE:u32 = 20;
const INSTRUCTIONS_PER_SEC:u16 = 600;
pub fn run() {
    let mut chip8 = Chip8::new();
    chip8.read_file("roms/2-ibm-logo.ch8");

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

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for color in color_buffer.iter_mut().take(3*64*32) {
            *color = 0;
        }

        chip8.step();

        for i in 0..64*32 {
            if chip8.screen_state[i] {
                color_buffer[i*3] = 255;
                color_buffer[i*3+1] = 100;
                color_buffer[i*3+2] = 100;
            }
        }

        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        texture.update(None, &color_buffer, 3 * 64).expect("Failed texture update");
        canvas.copy(&texture,None,None).unwrap();
        canvas.present();
        thread::sleep(Duration::from_millis(100));
    }
}

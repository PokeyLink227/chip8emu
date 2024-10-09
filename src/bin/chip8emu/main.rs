#![allow(unused_variables)]
extern crate sdl2;

use crate::chip8::{Chip8, Chip8Error};
use clap::Parser;
use sdl2::{event::Event, keyboard::Keycode, keyboard::Scancode, pixels::Color, rect::Rect};
use std::{fs::File, io::Read, time::Duration};
mod chip8;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Path to Rom to load into emulator
    #[arg(short, long, value_name = "rom")]
    filename: String,

    #[arg(short, long, value_name = "real pixels", default_value_t = 15)]
    pixel_width: u32,
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Chip-8 Emulator",
            64 * args.pixel_width,
            32 * args.pixel_width,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut emu = Chip8::new();
    let keybinds: [Scancode; 0x10] = [
        Scancode::Num0,
        Scancode::Num1,
        Scancode::Num2,
        Scancode::Num3,
        Scancode::Num4,
        Scancode::Num5,
        Scancode::Num6,
        Scancode::Num7,
        Scancode::Num8,
        Scancode::Num9,
        Scancode::A,
        Scancode::B,
        Scancode::C,
        Scancode::D,
        Scancode::E,
        Scancode::F,
    ];
    let _ = emu.load_rom(&args.filename, 0x200);

    let font_data: [u8; 50] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x60, 0xA0, 0x20, 0x20, 0xF0, // 1
        0x60, 0x90, 0x20, 0x40, 0xF0, // 2
        0xE0, 0x10, 0x60, 0x10, 0xE0, // 3
        0x90, 0x90, 0x60, 0x10, 0x10, // 4
        0xF0, 0x80, 0xE0, 0x10, 0xE0, // 5
        0x70, 0x80, 0xF0, 0x90, 0x60, // 6
        0xF0, 0x10, 0x20, 0x40, 0x80, // 7
        0x60, 0x90, 0x60, 0x90, 0x60, // 8
        0x60, 0x90, 0xF0, 0x10, 0x60, // 9
    ];
    emu.load_font(&font_data);

    'running: loop {
        // reset pressed keys
        let mut pressed = None;
        let mut step = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => step = true,
                Event::KeyUp { scancode: key, .. } => pressed = key,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        // draw emu output
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (y, row) in emu.get_pixels().iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                if *pixel {
                    canvas.fill_rect(Rect::new(
                        x as i32 * args.pixel_width as i32,
                        y as i32 * args.pixel_width as i32,
                        args.pixel_width,
                        args.pixel_width,
                    ))?;
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        // set pressed key
        emu.pressed_key = match pressed {
            None => None,
            Some(scancode) => match keybinds.iter().position(|x| *x == scancode) {
                None => None,
                Some(index) => Some(index as u8),
            },
        };

        // set keys that are down
        for (index, key) in keybinds.iter().enumerate() {
            emu.down_keys[index] = event_pump.keyboard_state().is_scancode_pressed(*key);
        }

        // clock cpu
        for _ in 0..10 {
            match emu.clock() {
                Ok(()) => {}
                Err(e) => println!("{:?}", e),
            }
        }
        emu.signal_new_frame();
    }

    Ok(())
}

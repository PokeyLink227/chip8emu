#![allow(unused_variables)]
extern crate sdl2;

use crate::chip8::{Chip8, Chip8Error};
use sdl2::{event::Event, keyboard::Keycode, keyboard::Scancode, pixels::Color, rect::Rect};
use std::{fs::File, io::Read, time::Duration};
mod chip8;

/*
#[derive(Debug)]
enum Instr {
    CallMachineCode(u16),
    ClearScreen,
    Return,
    Jump(u16),
    Call(u16),
    IfEqualImm(u8, u8),
    IfNotEqualImm(u8, u8),
    IfEqualReg(u8, u8),
    SetImm(u8, u8),
    AddImm(u8, u8),
    SetReg(u8, u8),
    OrReg(u8, u8),
    AndReg(u8, u8),
    XorReg(u8, u8),
    AddReg(u8, u8),
    SubReg(u8, u8),
    ShiftRight(u8, u8),
    SetSubReg(u8, u8),
    ShiftLeft(u8, u8),
    IfNotEqualReg(u8, u8),
    SetI(u16),
    JumpReg(u16),
    Rand(u8, u8),
    Draw(u8, u8, u8),
    IfKey(u8),
    IfNotKey(u8),
    GetTimer(u8),
    GetKey(u8),
    SetTimer(u8),
    SetSound(u8),
    AddIReg(u8),
    SetICharAddr(u8),
    StoreDecimal(u8),
    StoreReg(u8),
    LoadReg(u8),
    Error,
}
*/

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let cell_size = 15;

    let window = video_subsystem
        .window("Chip-8 Emulator", 64 * cell_size, 32 * cell_size)
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
    // y, x
    // emu.pixels[2][4] = true;

    //let _ = emu.load_rom("1-chip8-logo.ch8", 0x200);
    //let _ = emu.load_rom("2-ibm-logo.ch8", 0x200);
    //let _ = emu.load_rom("3-corax+.ch8", 0x200);
    //let _ = emu.load_rom("4-flags.ch8", 0x200);
    //let _ = emu.load_rom("5-quirks.ch8", 0x200);
    //let _ = emu.load_rom("6-keypad.ch8", 0x200);
    let _ = emu.load_rom("Soccer.ch8", 0x200);
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
                        x as i32 * cell_size as i32,
                        y as i32 * cell_size as i32,
                        cell_size,
                        cell_size,
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

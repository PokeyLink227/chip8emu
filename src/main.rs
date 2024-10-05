#![allow(unused_variables)]
extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use std::time::Duration;

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

#[derive(Debug)]
enum Chip8Error {
    InvalidInstruction,
    StackOverflow,
    StackUnderflow,
    AddressOverflow,
}

#[derive(Debug)]
enum Chip8Mode {
    Running,
    WaitingKey,
    Stopped,
}

#[derive(Debug)]
struct Chip8 {
    v: [u8; 0x10],
    pc: u16,
    i: u16,
    stack: [u16; 24],
    stack_pos: u8,
    delay_timer: u8,
    sound_timer: u8,
    memory: Vec<u8>,

    pixels: [[bool; 64]; 32],
    down_keys: [bool; 0x10],
    pressed_key: Option<u8>,
    mode: Chip8Mode,
}

impl Chip8 {
    fn new() -> Chip8 {
        Chip8 {
            v: [0; 0x10],
            pc: 0x200,
            i: 0x000,
            stack: [0x00; 24],
            stack_pos: 0,
            delay_timer: 0,
            sound_timer: 0,
            memory: vec![0; 0x1000],
            pixels: [[false; 64]; 32],
            pressed_key: None,
            down_keys: [false; 0x10],
            mode: Chip8Mode::Stopped,
        }
    }

    pub fn clock(&mut self) -> Result<(), Chip8Error> {
        let instr = self.fetch_instr();
        self.execute_instr(instr)
    }

    fn fetch_instr(&mut self) -> u16 {
        let instr: u16 = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.pc += 2;
        instr
    }

    fn execute_instr(&mut self, instr: u16) -> Result<(), Chip8Error> {
        let opcode: u8 = ((instr & 0xF000) >> (4 * 3)) as u8;
        let x: usize = ((instr & 0x0F00) >> (4 * 2)) as usize;
        let y: usize = ((instr & 0x00F0) >> (4 * 1)) as usize;
        let addr: u16 = instr & 0x0FFF;
        let imm_8: u8 = (instr & 0x00FF) as u8;
        let imm_4: u8 = (instr & 0x000F) as u8;

        match opcode {
            0x0 => match imm_8 {
                0xE0 => self.clear_screen(),
                // return
                0xEE => {
                    if self.stack_pos == 0 {
                        return Err(Chip8Error::StackUnderflow);
                    }

                    self.stack_pos -= 1;
                    self.pc = self.stack[self.stack_pos as usize];
                }
                _ => return Err(Chip8Error::InvalidInstruction),
            },
            // jump addr
            0x1 => {
                self.pc = addr;
            }
            // call addr
            0x2 => {
                if self.stack_pos as usize >= self.stack.len() {
                    return Err(Chip8Error::StackOverflow);
                }

                self.stack[self.stack_pos as usize] = self.pc;
                self.stack_pos += 1;
                self.pc = addr;
            }
            // if equal
            0x3 => {
                if self.v[x] == imm_8 {
                    self.pc += 2;
                }
            }
            // if not equal
            0x4 => {
                if self.v[x] != imm_8 {
                    self.pc += 2;
                }
            }
            // if equal
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // set reg
            0x6 => {
                self.v[x] = imm_8;
            }
            // add imm
            0x7 => {
                self.v[x] = self.v[x as usize].wrapping_add(imm_8);
            }
            0x8 => match imm_4 {
                0x0 => {
                    self.v[x] = self.v[y];
                }
                0x1 => {
                    self.v[x] |= self.v[y];
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                }
                0x4 => {
                    if self.v[x].checked_add(self.v[y]) == None {
                        self.v[0xF] = 0x01;
                    } else {
                        self.v[0xF] = 0x00;
                    }

                    self.v[x] = self.v[x].wrapping_add(self.v[y]);
                }
                0x5 => {
                    if self.v[y] > self.v[x] {
                        self.v[0xF] = 0x00;
                    } else {
                        self.v[0xF] = 0x01;
                    }

                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                }
                0x6 => {
                    self.v[0xF] = self.v[x] & 0x01;
                    self.v[x] >>= 1;
                }
                0x7 => {
                    if self.v[x] > self.v[y] {
                        self.v[0xF] = 0x00;
                    } else {
                        self.v[0xF] = 0x01;
                    }

                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                }
                0xE => {
                    self.v[0xF] = self.v[x] & 0x80;
                    self.v[x] <<= 1;
                }
                _ => return Err(Chip8Error::InvalidInstruction),
            },
            // if not equal
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => self.i = addr,
            // jump reg
            // TODO: wont overflow but will run out of bounds
            0xB => match addr.checked_add(self.v[0] as u16) {
                Some(new_addr) => self.pc = new_addr,
                None => return Err(Chip8Error::AddressOverflow),
            },
            // rand
            0xC => {
                todo!();
            }
            0xD => self.display_sprite(x, y, imm_4),
            0xE => match imm_8 {
                0x9E => {
                    if self.get_key_pressed(self.v[x]) {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.get_key_pressed(self.v[x]) {
                        self.pc += 2;
                    }
                }
                _ => return Err(Chip8Error::InvalidInstruction),
            },
            0xF => match imm_8 {
                0x07 => self.v[x] = self.delay_timer,
                0x0A => match self.get_next_key() {
                    Some(key) => self.v[x] = key,
                    None => self.pc -= 2,
                },
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i += self.v[x] as u16,
                0x29 => self.i = self.get_sprite_addr(self.v[x]),
                0x33 => todo!(),
                0x55 => {
                    for offset in 0..x {
                        let effective_addr = self.i as usize + offset;
                        if (effective_addr & 0xF000) != 0x0000 {
                            return Err(Chip8Error::AddressOverflow);
                        }
                        self.memory[effective_addr] = self.v[offset];
                    }
                }
                _ => return Err(Chip8Error::InvalidInstruction),
            },
            _ => return Err(Chip8Error::InvalidInstruction),
        }

        Ok(())
    }

    fn clear_screen(&mut self) {
        for row in &mut self.pixels {
            for pix in row {
                *pix = false;
            }
        }
    }

    fn display_sprite(&mut self, x: usize, y: usize, ofset: u8) {}

    fn get_key_pressed(&self, key: u8) -> bool {
        self.down_keys[key as usize]
    }

    fn get_next_key(&self) -> Option<u8> {
        self.pressed_key
    }

    fn get_sprite_addr(&self, index: u8) -> u16 {
        0
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 64, 32)
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

    'running: loop {
        // reset pressed keys
        emu.pressed_key = None;
        let mut pressed = None;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { scancode: key, .. } => pressed = key,
                _ => {}
            }
        }

        canvas.clear();
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));

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
        let _ = emu.clock();

        if emu.down_keys[0] {
            canvas.set_draw_color(Color::RGB(255, 0, 0));
        } else {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
        }
    }

    Ok(())
}
/*
fn main() {
    let mut emu: Chip8 = Chip8::new();

    println!("{:?}", emu);
    let _ = emu.clock();
    println!("{:?}", emu);
}
*/

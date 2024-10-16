use rand::random;
use std::{fs::File, io::Read};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Chip8Error {
    InvalidInstruction,
    StackOverflow,
    StackUnderflow,
    AddressOverflow,
    BadRomPath,
    IOError,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Chip8Mode {
    Running,
    WaitingKey,
    Stopped,
}

pub struct Chip8 {
    v: [u8; 0x10],
    pc: u16,
    i: u16,
    stack: [u16; 24],
    stack_pos: u8,
    delay_timer: u8,
    sound_timer: u8,
    memory: Vec<u8>,

    pixels: [[bool; 64]; 32],
    pub down_keys: [bool; 0x10],
    pub pressed_key: Option<u8>,
    sprite_drawn: bool,
    pub mode: Chip8Mode,
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
            sprite_drawn: false,
            mode: Chip8Mode::Stopped,
        }
    }

    pub fn clock(&mut self) -> Result<(), Chip8Error> {
        //print!("{}: ", self.pc);
        let instr = self.fetch_instr()?;
        //println!("{}", instr);
        self.execute_instr(instr)
    }

    pub fn load_rom(&mut self, filename: &str, address: u16) -> Result<(), Chip8Error> {
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(_) => return Err(Chip8Error::BadRomPath),
        };
        match file.read(&mut self.memory[0x200..]) {
            Ok(_) => Ok(()),
            Err(e) => Err(Chip8Error::IOError),
        }
    }

    pub fn load_font(&mut self, font_data: &[u8; 50]) {
        self.memory[0..50].copy_from_slice(font_data);
    }

    pub fn get_pixels(&self) -> &[[bool; 64]; 32] {
        &self.pixels
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn signal_new_frame(&mut self) {
        self.sprite_drawn = false;
        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
    }

    fn fetch_instr(&mut self) -> Result<u16, Chip8Error> {
        if self.pc & 0xF000 != 0x0000 {
            return Err(Chip8Error::AddressOverflow);
        }

        let instr: u16 = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);

        self.pc += 2;
        Ok(instr)
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
                0x01 => self.mode = Chip8Mode::Stopped,
                _ => return Err(Chip8Error::InvalidInstruction),
            },
            // jump addr
            0x1 => {
                // can never be out of bounds
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
                    self.v[0xF] = 0x00;
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.v[0xF] = 0x00;
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.v[0xF] = 0x00;
                }
                0x4 => {
                    let flag = if self.v[x].checked_add(self.v[y]) == None {
                        0x01
                    } else {
                        0x00
                    };

                    self.v[x] = self.v[x].wrapping_add(self.v[y]);
                    self.v[0xF] = flag;
                }
                0x5 => {
                    let flag = if self.v[y] > self.v[x] { 0x00 } else { 0x01 };

                    self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    self.v[0xF] = flag;
                }
                0x6 => {
                    // chip8 quirk: requires x = y >> 1
                    let flag = self.v[y] & 0x01;
                    self.v[x] = self.v[y] >> 1;
                    self.v[0xF] = flag;
                }
                0x7 => {
                    let flag = if self.v[x] > self.v[y] { 0x00 } else { 0x01 };

                    self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    self.v[0xF] = flag;
                }
                0xE => {
                    // chip8 quirk: requires x = y << 1
                    let flag = (self.v[y] & 0x80) >> 7;
                    self.v[x] = self.v[y] << 1;
                    self.v[0xF] = flag;
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
            0xB => {
                self.pc += addr + self.v[0] as u16;
                if self.pc & 0xF000 != 0x0000 {
                    return Err(Chip8Error::AddressOverflow);
                }
            }
            // rand
            0xC => self.v[x] = random::<u8>() & imm_8,
            0xD => self.display_sprite(self.v[x] as usize, self.v[y] as usize, imm_4),
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
                    Some(key) => {
                        self.v[x] = key;
                        self.pressed_key = None;
                    }
                    None => self.pc -= 2,
                },
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x29 => self.i = self.get_sprite_addr(self.v[x]),
                0x33 => {
                    if (self.i + 2) & 0xF000 != 0x0000 {
                        return Err(Chip8Error::AddressOverflow);
                    }

                    self.memory[self.i as usize + 0] = self.v[x] / 100;
                    self.memory[self.i as usize + 1] = self.v[x] % 100 / 10;
                    self.memory[self.i as usize + 2] = self.v[x] % 10;
                }
                0x55 => {
                    if (self.i + x as u16) & 0xF000 != 0x0000 {
                        return Err(Chip8Error::AddressOverflow);
                    }

                    for offset in 0..=x {
                        let effective_addr = self.i as usize + offset;
                        self.memory[effective_addr] = self.v[offset];
                    }
                    // chip8 quirk
                    self.i += x as u16 + 1;
                }
                0x65 => {
                    if (self.i + x as u16) & 0xF000 != 0x0000 {
                        return Err(Chip8Error::AddressOverflow);
                    }

                    for offset in 0..=x {
                        let effective_addr = self.i as usize + offset;
                        self.v[offset] = self.memory[effective_addr];
                    }
                    // chip8 quirk
                    self.i += x as u16 + 1;
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

    fn display_sprite(&mut self, x: usize, y: usize, size: u8) {
        if self.sprite_drawn {
            self.pc -= 2;
            return;
        }
        self.sprite_drawn = true;

        let mut collision: u8 = 0;
        let x = x % 64;
        let y = y % 32;

        for row in 0..(size as usize) {
            if y + row >= 32 {
                break;
            }

            let sprite = self.memory[row + self.i as usize];
            for bit_index in 0..8 {
                if x + bit_index >= 64 {
                    break;
                }

                if self.pixels[(y + row) % 32][(x + bit_index) % 64]
                    && (sprite << bit_index) & 0x80 == 0x80
                {
                    collision = 1;
                }
                self.pixels[(y + row) % 32][(x + bit_index) % 64] ^=
                    (sprite << bit_index) & 0x80 == 0x80;
            }
        }
        self.v[0xF] = collision;
    }

    fn get_key_pressed(&self, key: u8) -> bool {
        self.down_keys[key as usize]
    }

    // returns the next key pressed and released
    fn get_next_key(&self) -> Option<u8> {
        self.pressed_key
    }

    fn get_sprite_addr(&self, index: u8) -> u16 {
        // each character takes up 5 bytes
        // character sprites are stored starting at address 0
        (index as u16) * 5
    }
}

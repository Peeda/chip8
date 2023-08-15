use std::io::prelude::*;
use std::cmp;
use std::fs::File;
use std::mem;

use rand::Rng;
const RAM_SIZE:usize = 4096;
const SCREEN_WIDTH:usize = 64;
const SCREEN_HEIGHT:usize = 32;
const REGISTERS:usize = 16;
const FONT_DATA:[u8;16*5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
const PROGRAM_START:usize = 0x200;
const FONT_START:usize = 0x050;
#[derive(PartialEq)]
pub enum Version {
    Original,
    Schip,
    XoChip,
}
pub struct Chip8 {
    ram:[u8;RAM_SIZE],
    pc:u16,
    pub screen_state:[bool;SCREEN_WIDTH*SCREEN_HEIGHT],
    i_reg:u16,
    v_reg:[u8;REGISTERS],
    stack:Vec<u16>,
    pub delay_timer:u8,
    pub sound_timer:u8,
    pub file_loaded:bool,
    pub input_data:[bool;16],
    pub prev_input_data:[bool;16],
    pub screen_updated:bool,
    version:Version,
}
impl Default for Chip8 {
    fn default() -> Chip8 {
        Chip8::new(Version::Schip)
    }
}
impl Chip8 {
    pub fn new(version:Version) -> Chip8 {
        let mut chip8 = Chip8 {
            ram:[0;RAM_SIZE],
            pc:PROGRAM_START as u16,
            screen_state:[false;SCREEN_WIDTH*SCREEN_HEIGHT],
            i_reg:0,
            v_reg:[0;REGISTERS],
            stack:Vec::new(),
            delay_timer:0,
            sound_timer:0,
            file_loaded:false,
            input_data:[false;16],
            prev_input_data:[false;16],
            version,
            screen_updated:false,
        };
        for (i,font_byte) in FONT_DATA.iter().enumerate() {
            chip8.ram[FONT_START + i] = *font_byte;
        }
        chip8
    }
    pub fn read_file(&mut self, path:&str) {
        if !self.file_loaded {
            let rom:File = File::open(path).expect("Failed to read file");
            let mut ind = PROGRAM_START;
            for byte in rom.bytes() {
                self.ram[ind] = byte.expect("Failed to read byte from file");
                ind += 1;
            }
            self.file_loaded = true;
        } else {println!("tried to read file after load")}
    }
    pub fn step(&mut self) {
        self.screen_updated = false;
        let higher_byte:u16 = self.ram[self.pc as usize] as u16;
        let lower_byte:u16 = self.ram[(self.pc + 1) as usize] as u16;
        let opcode:u16 = higher_byte << 8 | lower_byte;
        let nibbles = ((opcode & 0xF000) >> 12, (opcode & 0x0F00) >> 8, 
                       (opcode & 0x00F0) >> 4, opcode & 0x000F);
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;
        self.pc += 2;
        match nibbles {
            (0,0,0xE,0) => {
                for bit in self.screen_state.iter_mut() {
                    *bit = false;
                }
            }
            (0,0,0xE,0xE) => self.pc = self.stack.pop().expect("Tried to pop empty stack"),
            (1,_,_,_) => self.pc = nnn,
            (2,_,_,_) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            (3,_,_,_) => if self.v_reg[x] == nn {self.pc += 2},
            (4,_,_,_) => if self.v_reg[x] != nn {self.pc += 2},
            (5,_,_,0) => if self.v_reg[x] == self.v_reg[y] {self.pc += 2},
            (6,_,_,_) => self.v_reg[x] = nn,
            (7,_,_,_) => self.v_reg[x] = self.v_reg[x].wrapping_add(nn),
            (8,_,_,0) => self.v_reg[x] = self.v_reg[y],
            (8,_,_,1|2|3) => {
                match nibbles.3 {
                    1 => self.v_reg[x] |= self.v_reg[y],
                    2 => self.v_reg[x] &= self.v_reg[y],
                    3 => self.v_reg[x] ^= self.v_reg[y],
                    _ => ()
                }
                if self.version == Version::Original {
                    self.v_reg[0xF_usize] = 0
                }
            }
            (8,_,_,4) => {
                let (sum,overflow) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                self.v_reg[x] = sum;
                if overflow {
                    self.v_reg[0xF_usize] = 1;
                } else {
                    self.v_reg[0xF_usize] = 0;
                }
            }
            (8,_,_,5 | 7) => {
                let mut first = self.v_reg[x];
                let mut second = self.v_reg[y];
                if nibbles.3 == 7 {
                    mem::swap(&mut first, &mut second);
                }
                let (difference,underflow) = first.overflowing_sub(second);
                self.v_reg[x] = difference;
                if underflow {
                    self.v_reg[0xF_usize] = 0;
                } else {
                    self.v_reg[0xF_usize] = 1;
                }
            }
            (8,_,_,6 | 0xE) => {
                match self.version {
                    Version::Original | Version::XoChip => self.v_reg[x] = self.v_reg[y],
                    Version::Schip => (),
                }
                match nibbles.3 {
                    6 => {
                        let temp = self.v_reg[x] & 1;
                        self.v_reg[x] >>= 1;
                        self.v_reg[0xF_usize] = temp;
                    }
                    0xE => {
                        let temp = if self.v_reg[x] & (1 << 7) > 0 {
                            1
                        } else {
                            0
                        };
                        self.v_reg[x] <<= 1;
                        self.v_reg[0xF_usize] = temp;
                    }
                    _ => ()
                }
            }
            (9,_,_,0) => if self.v_reg[x] != self.v_reg[y] {self.pc += 2}
            (0xA,_,_,_) => self.i_reg = nnn,
            (0xB,_,_,_) => {
                let add:u16 = match self.version {
                    Version::Original | Version::XoChip => self.v_reg[0] as u16,
                    Version::Schip => self.v_reg[x] as u16,
                };
                self.pc = nnn + add;
            }
            (0xC,_,_,_) => {
                let mut rng = rand::thread_rng();
                let rand_num:u8 = rng.gen();
                self.v_reg[x] = rand_num & nn;
            }
            (0xD,_,_,_) => {
                const WIDTH:u8 = 8;
                self.v_reg[0xF_usize] = 0;
                let img_x = self.v_reg[x] % 64;
                let img_y = self.v_reg[y] % 64;

                let right_end = cmp::min(n + img_y,SCREEN_HEIGHT as u8);
                let bottom_end = cmp::min(WIDTH + img_x,SCREEN_WIDTH as u8);
                for row in img_y..right_end {
                    let sprite_addr = (self.i_reg + row as u16 - img_y as u16) as usize;
                    let row_data:u8 = self.ram[sprite_addr];
                    for col in img_x..bottom_end{
                        let ind:usize = (row as u16 * SCREEN_WIDTH as u16 + col as u16) as usize;
                        let shift = (WIDTH - 1) - (col - img_x);
                        if row_data & (1 << shift) > 0 {
                            if self.screen_state[ind] {
                                self.screen_state[ind] = false;
                                self.v_reg[0xF_usize] = 1;
                            } else {
                                self.screen_state[ind] = true;
                            }
                        }
                    }
                }
                self.screen_updated = true;
            }
            (0xE,_,9,0xE) => if self.input_data[self.v_reg[x] as usize] {self.pc += 2},
            (0xE,_,0xA,1) => if !self.input_data[self.v_reg[x] as usize] {self.pc += 2},
            (0xF,_,0,7) => self.v_reg[x] = self.delay_timer,
            (0xF,_,0,0xA) => {
                //this one activates on release
                let mut keycode:Option<u8> = None;
                for (ind,held) in self.input_data.iter().enumerate() {
                    if self.prev_input_data[ind] && !(*held) {
                        keycode = Some(ind.try_into().expect("keycode > 15"));
                    }
                }
                if let Some(k) = keycode {
                    self.v_reg[x] = k;
                } else {
                    self.pc -= 2;
                }
            }
            (0xF,_,1,5) => self.delay_timer = self.v_reg[x],
            (0xF,_,1,8) => self.sound_timer = self.v_reg[x],
            (0xF,_,1,0xE) => self.i_reg += self.v_reg[x] as u16,
            (0xF,_,2,9) => {
                let nibble:u8 = self.v_reg[x] & 0xF;
                self.i_reg = FONT_START as u16 + 5_u16 * nibble as u16;
            }
            (0xF,_,3,3) => {
                let mut val = self.v_reg[x];
                let ones = val % 10;
                val -= ones;
                let tens = val % 100;
                let hundreds = val - tens;
                assert!(tens % 10 == 0);
                assert!(hundreds % 100 == 0);
                self.ram[self.i_reg as usize] = hundreds/100;
                self.ram[(self.i_reg + 1) as usize] = tens/10;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            (0xF,_,5 | 6,5) => {
                for i in 0..(x+1) {
                    if nibbles.2 == 5 {
                        self.ram[self.i_reg as usize + i] = self.v_reg[i];
                    } else {
                        self.v_reg[i] = self.ram[self.i_reg as usize + i];
                    }
                }
                match self.version {
                    Version::Original | Version::XoChip => self.i_reg += (x+1) as u16,
                    Version::Schip => (),
                }
            }
            _ => println!("unimplemented opcode {:#06X}", opcode),
        }
    }
    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 { 
            self.sound_timer -= 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_read() {
        const LOGO: [u8; 0x104] = [
            0x00, 0xE0, 0x61, 0x01, 0x60, 0x08, 0xA2, 0x50, 0xD0, 0x1F, 0x60, 0x10, 0xA2, 0x5F, 0xD0, 0x1F, 
            0x60, 0x18, 0xA2, 0x6E, 0xD0, 0x1F, 0x60, 0x20, 0xA2, 0x7D, 0xD0, 0x1F, 0x60, 0x28, 0xA2, 0x8C, 
            0xD0, 0x1F, 0x60, 0x30, 0xA2, 0x9B, 0xD0, 0x1F, 0x61, 0x10, 0x60, 0x08, 0xA2, 0xAA, 0xD0, 0x1F, 
            0x60, 0x10, 0xA2, 0xB9, 0xD0, 0x1F, 0x60, 0x18, 0xA2, 0xC8, 0xD0, 0x1F, 0x60, 0x20, 0xA2, 0xD7, 
            0xD0, 0x1F, 0x60, 0x28, 0xA2, 0xE6, 0xD0, 0x1F, 0x60, 0x30, 0xA2, 0xF5, 0xD0, 0x1F, 0x12, 0x4E, 
            0x0F, 0x02, 0x02, 0x02, 0x02, 0x02, 0x00, 0x00, 0x1F, 0x3F, 0x71, 0xE0, 0xE5, 0xE0, 0xE8, 0xA0, 
            0x0D, 0x2A, 0x28, 0x28, 0x28, 0x00, 0x00, 0x18, 0xB8, 0xB8, 0x38, 0x38, 0x3F, 0xBF, 0x00, 0x19, 
            0xA5, 0xBD, 0xA1, 0x9D, 0x00, 0x00, 0x0C, 0x1D, 0x1D, 0x01, 0x0D, 0x1D, 0x9D, 0x01, 0xC7, 0x29, 
            0x29, 0x29, 0x27, 0x00, 0x00, 0xF8, 0xFC, 0xCE, 0xC6, 0xC6, 0xC6, 0xC6, 0x00, 0x49, 0x4A, 0x49, 
            0x48, 0x3B, 0x00, 0x00, 0x00, 0x01, 0x03, 0x03, 0x03, 0x01, 0xF0, 0x30, 0x90, 0x00, 0x00, 0x80, 
            0x00, 0x00, 0x00, 0xFE, 0xC7, 0x83, 0x83, 0x83, 0xC6, 0xFC, 0xE7, 0xE0, 0xE0, 0xE0, 0xE0, 0x71, 
            0x3F, 0x1F, 0x00, 0x00, 0x07, 0x02, 0x02, 0x02, 0x02, 0x39, 0x38, 0x38, 0x38, 0x38, 0xB8, 0xB8, 
            0x38, 0x00, 0x00, 0x31, 0x4A, 0x79, 0x40, 0x3B, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 
            0x00, 0x00, 0xA0, 0x38, 0x20, 0xA0, 0x18, 0xCE, 0xFC, 0xF8, 0xC0, 0xD4, 0xDC, 0xC4, 0xC5, 0x00, 
            0x00, 0x30, 0x44, 0x24, 0x14, 0x63, 0xF1, 0x03, 0x07, 0x07, 0x77, 0x57, 0x53, 0x71, 0x00, 0x00, 
            0x28, 0x8E, 0xA8, 0xA8, 0xA6, 0xCE, 0x87, 0x03, 0x03, 0x03, 0x87, 0xFE, 0xFC, 0x00, 0x00, 0x60, 
            0x90, 0xF0, 0x80, 0x70
        ];
        let mut hard_coded = Chip8::default();
        let mut from_file = Chip8::default();
        for i in 0..LOGO.len() {
            hard_coded.ram[PROGRAM_START + i] = LOGO[i];
        }
        from_file.read_file("roms/1-chip8-logo.ch8");
        for i in 0..RAM_SIZE {
            assert_eq!(hard_coded.ram[i],from_file.ram[i]);
        }
    }
    #[test]
    fn test_font_data() {
        for i in 0..16 {
            let mut chip8 = Chip8::default();
            //call FX29 and point to char i
            chip8.v_reg[0] = i;
            //make the upper nibble point to register 0 each time
            chip8.ram[PROGRAM_START] = 0xF0;
            chip8.ram[PROGRAM_START + 1] = 0x29;
            chip8.step();
            //assert that location of I matches the font data
            for k in 0..5 {
                assert_eq!(chip8.ram[chip8.i_reg as usize + k],
                           FONT_DATA[i as usize * 5 + k]);
            }
        }
    }
}

extern crate rand;

use rand::Rng;

use std::fs;

#[derive(Debug)]
enum Opcode {
    ClearScreen,                // 00E0
    Return,                     // 00EE
    Jump(u16),                  // 1nnn
    Call(u16),                  // 2nnn
    Skip(u16,u8),               // 3xkk
    SkipNotEqual(u16,u8),       // 4xkk
    SkipRegEqual(u16,u16),      // 5xy0
    Set(u16,u8),              // 6xkk
    IncrementReg(u16,u8),     // 7xkk
    CopyReg(u16,u16),           // 8xy0
    BitwiseOr(u16,u16),         // 8xy1
    BitwiseAnd(u16,u16),        // 8xy2
    BitwiseXor(u16,u16),        // 8xy3

    AddReg(u16,u16),                // 8xy4
    SubtractReg(u16,u16),           // 8xy5
    BitwiseRight(u16, u16),         // 8xy6
    NegativeSubtractReg(u16, u16),  // 8xy7
    BitwiseLeft(u16,u16),           // 8xyE

    SkipRegNotEqual(u16,u16),       // 9xy0
    SetI(u16),                      // Annn
    JumpOffset(u16),                // Bnnn
    RandomAnd(u16, u8),             // Cxkk
    Draw(u8,u8,u8),                 // Dxyn
    SkipKeyPressed(u16),            // Ex9E
    SkipKeyNotPressed(u16),         // ExA1
    CopyDelayToReg(u8),             // Fx07
    WaitForKeyPress(u16),           // Fx0A
    SetDelayFromReg(u16),           // Fx15
    SetSoundFromReg(u16),           // Fx18
    AddI(u16),                      // Fx1E
    SetIToFontDigit(u16),           // Fx29
    BinaryCodeI(u16),               // Fx33
    CopyRegistersToI(u16),          // Fx55
    CopyIToRegisters(u16),          // Fx65
}

struct Display {
    pixels: [[bool;64];32]
}

impl Display {
    fn new() -> Display {
        Display {
            pixels: [[false; 64];32]
        }
    }

    fn clear(&mut self) {
        self.pixels = [[false; 64];32];
    }

    fn draw(&self) {
        print!("{}[2J", 27 as char);
        for row in self.pixels {
            for pixel in row {
                if pixel {
                    print!("▉"); //("█")
                }
                else {
                    print!(" ");
                }
            }
            println!("");
        }
    }
}

struct Emulator {
    V: [u8; 16],
    memory: Vec<u8>,
    register: u16,
    pc: u16,
    I: u16,
    stack: Vec<u16>,
    sp: u16,
    display: Display,
    sound_timer: u8,
    delay_timer: u8,
    draw_flag: bool
}

impl Emulator {
    fn new() -> Self {

        let mut emulator = Emulator {
            V: [0; 16],
            memory: vec![0; 4096],
            register: 0,
            pc: 0,
            I: 0,
            stack: Vec::new(),
            sp: 0,
            display: Display::new(),
            sound_timer: 0,
            delay_timer: 0,
            draw_flag: false
        };

        let fontset = Emulator::get_fontset();

        emulator.memory[0x50..(0x50+fontset.len())].clone_from_slice(&fontset);

        //println!("{:02x?}", emulator.memory);

        emulator
    }

    fn load(&mut self, prog: Vec<u8>) {
        self.memory[0x200..(0x200+prog.len())].clone_from_slice(&prog[0..prog.len()]);
        self.pc = 0x200;
    }

    
    fn fetch(&mut self) -> u16 {
        let opcode1 = self.memory[(self.pc as usize)] as u16;
        self.pc += 1;
        let opcode2 = self.memory[(self.pc as usize)] as u16;
        self.pc += 1;
        let opcode = opcode1 << 8 | opcode2;
        opcode
    }
    
    fn decode(instruction: u16) -> Opcode {
        match instruction {
            0x00e0 => Opcode::ClearScreen,
            0x1000..=0x1fff => Opcode::Jump(instruction& 0x0fff),
            0x2000..=0x2fff => Opcode::Call(instruction& 0x0fff),
            0x3000..=0x3fff => {
                let reg = (instruction & 0x0f00) >> 8;
                let val = (instruction & 0x00ff) as u8;
                Opcode::Skip(reg, val)
            },
            0x4000..=0x4fff => {
                let reg = (instruction & 0x0f00) >> 8;
                let val = (instruction & 0x00ff) as u8;
                Opcode::SkipNotEqual(reg, val)
            },
            0x5000..=0x5ff0 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::SkipRegEqual(reg1, reg2)
            },
            0x6000..=0x6fff => {
                let reg = (instruction & 0x0f00) >> 8;
                let value = (instruction & 0x00ff) as u8;
                Opcode::Set(reg, value)
            },
            0x7000..=0x7fff => {
                let reg = (instruction & 0x0f00) >> 8;
                let value = (instruction & 0x00ff) as u8;
                Opcode::IncrementReg(reg, value)
            },
            0x8000..=0x8ff0 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::CopyReg(reg1, reg2)
            },
            0x8001..=0x8ff1 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::BitwiseOr(reg1, reg2)
            },
            0x8002..=0x8ff2 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::BitwiseAnd(reg1, reg2)
            }, 
            0x8003..=0x8ff3 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::BitwiseXor(reg1, reg2)
            },
            0x8004..=0x8ff4 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::AddReg(reg1, reg2)
            },
            0x8005..=0x8ff5 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::SubtractReg(reg1, reg2)
            },
            0x8005..=0x8ff5 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::SubtractReg(reg1, reg2)
            },
            0x8005..=0x8ff6 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::BitwiseRight(reg1, reg2)
            },
            0x8005..=0x8ff7 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::NegativeSubtractReg(reg1, reg2)
            },
            0x800e..=0x8ffe => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::BitwiseLeft(reg1, reg2)
            },                                                   
            0x9000..=0x9ff0 => {
                let reg1 = (instruction & 0x0f00) >> 8;
                let reg2 = (instruction & 0x00f0) >> 4;
                Opcode::SkipRegNotEqual(reg1, reg2)
            },            
            0xa000..=0xafff => Opcode::SetI(instruction & 0x0fff),
            0xb000..=0xbfff => Opcode::JumpOffset(instruction & 0x0fff),
            0xc000..=0xcfff => {
                let reg = (instruction & 0x0f00) >> 8;
                let value = (instruction & 0x00ff) as u8;
                Opcode::RandomAnd(reg, value)
            },
            0xd000..=0xdfff => {
                let x = (instruction & 0x0f00) >> 8;
                let y = (instruction & 0x00f0) >> 4;
                let rows = (instruction & 0x000f) as u8;
                Opcode::Draw(x as u8,y as u8, rows)
            },
            0xe09e..=0xef9e => Opcode::SkipKeyPressed((instruction & 0x0f00) >> 8),
            0xe0a1..=0xefa1 => Opcode::SkipKeyNotPressed((instruction & 0x0f00) >> 8),
            0xf007..=0xff07 => Opcode::CopyDelayToReg(((instruction & 0x0f00) >> 8) as u8),
            0xf00a..=0xff0a => Opcode::WaitForKeyPress((instruction & 0x0f00) >> 8),
            0xf015..=0xff15 => Opcode::SetDelayFromReg((instruction & 0x0f00) >> 8),
            0xf018..=0xff18 => Opcode::SetSoundFromReg((instruction & 0x0f00) >> 8),
            0xf01e..=0xff1e => Opcode::AddI((instruction & 0x0f00) >> 8),
            0xf029..=0xff29 => Opcode::SetIToFontDigit((instruction & 0x0f00) >> 8),
            0xf033..=0xff33 => Opcode::BinaryCodeI((instruction & 0x0f00) >> 8),
            // fx29 set i = location of sprite goes here
            // fx33 bcd decode goes here
            0xf055..=0xff55 => Opcode::CopyRegistersToI((instruction & 0x0f00) >> 8),
            0xf065..=0xff65 => Opcode::CopyIToRegisters((instruction & 0x0f00) >> 8),
            _ => panic!("Oh no!")
        }
    }
    
    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => {
                self.display.clear();
                self.draw_flag = true;
            },
            Opcode::Jump(addr) => {
                self.pc = addr;
            },
            Opcode::Call(addr) => {
                self.push_stack(self.pc);
                self.pc = addr;
            },
            Opcode::Skip(reg,val) => {
                if val == self.V[reg as usize] {
                    self.pc += 2;
                }
            },
            Opcode::SkipNotEqual(reg,val) => {
                if val != self.V[reg as usize] {
                    self.pc += 2;
                }
            },
            Opcode::SkipRegEqual(reg1,reg2) => {
                if self.V[reg1 as usize] == self.V[reg2 as usize] {
                    self.pc += 2;
                }
            },                     
            Opcode::Set(v, n) => self.V[v as usize] = n,
            Opcode::IncrementReg(v, n) => self.V[v as usize] += n,
            Opcode::CopyReg(reg1, reg2) => self.V[reg1 as usize] = self.V[reg2 as usize],
            Opcode::BitwiseOr(reg1, reg2) => self.V[reg1 as usize] |= self.V[reg2 as usize],
            Opcode::BitwiseAnd(reg1, reg2) => self.V[reg1 as usize] &= self.V[reg2 as usize],
            Opcode::BitwiseXor(reg1, reg2) => self.V[reg1 as usize] ^= self.V[reg2 as usize],
            Opcode::AddReg(reg1, reg2) => {
                let sum = self.V[reg1 as usize] as usize + self.V[reg2 as usize] as usize;
                if sum > 255 {
                    self.V[15] = 1;
                    self.V[reg1 as usize] = (sum & 0b100000000) as u8;
                }
                else {
                    self.V[reg1 as usize] = sum as u8;
                }
            },
            Opcode::SubtractReg(reg1, reg2) => {
                let v1 = self.V[reg1 as usize];
                let v2 = self.V[reg2 as usize];
                if v1 > v2 {
                    self.V[reg1 as usize] = v1 - v2;
                    self.V[15] = 1;
                }
                else {
                    self.V[15] = 0;
                }
            },
            Opcode::BitwiseRight(reg1, _) => {
                let least_bit = (self.V[reg1 as usize] & 1) as u8;
                self.V[15] = least_bit;
                self.V[reg1 as usize] >>= 1;
            },
            Opcode::NegativeSubtractReg(reg1, reg2) => {
                let v1 = self.V[reg1 as usize];
                let v2 = self.V[reg2 as usize];
                if v2 > v1 {
                    self.V[reg1 as usize] = v2 - v1;
                    self.V[15] = 1;
                }
                else {
                    self.V[15] = 0;
                }                
            },
            Opcode::BitwiseLeft(reg1, _) => {
                let most_significant_bit = (self.V[reg1 as usize] & 0b10000000) >> 7;
                self.V[15] = most_significant_bit;
                self.V[reg1 as usize] <<= 1;                
            },
            Opcode::SkipRegNotEqual(reg1,reg2) => {
                if self.V[reg1 as usize] != self.V[reg2 as usize] {
                    self.pc += 2;
                }
            },              
            Opcode::SetI(i) => self.I = i,
            Opcode::JumpOffset(offset) => {
                self.pc = self.V[0] as u16 + offset;
            },
            Opcode::RandomAnd(reg, val) => {
                let rando: u8 = rand::thread_rng().gen();
                self.V[reg as usize] = rando & val;
            },
            Opcode::Draw(x, y, rows) => {
                let vx = self.V[x as usize] % 64;
                let vy = self.V[y as usize] % 32;
                //println!("row count = {}", rows);
                //let ourrange = &self.memory[self.I as usize..=(self.I as usize+rows as usize)];
                //println!("I-slice = {:04x?}", ourrange);                
                for i in 0..rows {
                    let row_byte = self.memory[self.I as usize +i as usize];
                    for j in 0..8 {
                        let x_coord = ((vx + j) % 64) as usize;
                        let y_coord = ((vy + i) % 32) as usize;
                        self.V[15] = 0;
                        let bit_value = ( row_byte >> (7-j) ) & 1;
                        //println!("i = {}, j = {}, x = {}, y = {}, row_byte = {}, bit = {}", 
                        //    i, j, x_coord, y_coord, row_byte, bit_value);
                        let screen_pixel = &mut (self.display.pixels[y_coord][x_coord]);
                        if bit_value == 1  {
                            if *screen_pixel == true {
                                *screen_pixel = false;
                                self.V[15] = 1;
                            }
                            else { 
                                *screen_pixel = true;
                            }
                        }
                    }
                }

                self.draw_flag = true;
            },
            Opcode::SkipKeyPressed(reg1) => (),
            Opcode::SkipKeyNotPressed(reg1) => (),
            Opcode::CopyDelayToReg(reg1) => self.V[reg1 as usize] = self.delay_timer,
            Opcode::WaitForKeyPress(reg1) => (),
            Opcode::SetDelayFromReg(reg1) => self.delay_timer = self.V[reg1 as usize],
            Opcode::SetSoundFromReg(reg1) => self.sound_timer = self.V[reg1 as usize],
            Opcode::AddI(val) => self.I += val,
            Opcode::SetIToFontDigit(reg) => {
                let val = self.V[reg as usize];
                self.I = 0x50 + val as u16;
            },
            Opcode::BinaryCodeI(reg) => {
                let val_string = format!("{:03}",self.V[reg as usize]);
                let digits: Vec<u8> = val_string
                    .chars()
                    .map(|d| d.to_digit(10).unwrap() as u8)
                    .collect();

                self.memory[self.I as usize] = digits[0];
                self.memory[(self.I+1) as usize] = digits[1];
                self.memory[(self.I+2) as usize] = digits[2];

            },
            Opcode::CopyRegistersToI(n) => {
                for i in 0..=n {
                    self.memory[(self.I+i) as usize] = self.V[i as usize];
                }
            },
            Opcode::CopyIToRegisters(n) => {
                for i in 0..=n {
                    self.V[i as usize] = self.memory[(self.I+i) as usize];
                }
            },
            _ => panic!("Oh no!")
        }
    }

    fn push_stack(&mut self, val: u16) {
        self.stack.push(val);
        self.sp += 1;
    }

    fn pop_stack(&mut self) -> u16 {
        self.sp -=1;
        self.stack.pop().unwrap()
    }

    fn get_fontset() -> [u8; 80] {
        let fontset: [u8; 80] =
        [   0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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

        fontset
    }

    fn run(&mut self) {
        //todo!()
        // while true
        // fetch
        // decode
        // execute
        while true {
            let opcode = self.fetch();
            let instruction = Self::decode(opcode);
            if opcode == 0x0 { break; }
            //println!("{:04x}: Opcode =      {:04x?}", self.pc-2, opcode);
            //println!("{:04x}: Instruction = {:?}", self.pc-2, instruction);
            self.execute(instruction);
            if (self.draw_flag) {
                self.draw();
            }

            if opcode == 0xd01f {
                //panic!("Bail!");
            }
        }    
    }

    fn draw(&mut self) {
        self.display.draw();
        self.draw_flag = false;
    }
}

fn main() {
    let mut emu = Emulator::new();
    let data = load("programs/ibm_logo.ch8");
    println!("{:02x?}", data);
    emu.load(data);
    emu.run();
}

fn load(filename: &str) ->Vec<u8> {
    let data = fs::read(filename).expect("Unable to read file");
    data
}

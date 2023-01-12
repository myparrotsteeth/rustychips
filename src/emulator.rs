mod display;
mod keypad;

use crate::emulator::keypad::Keypad;
use crate::emulator::display::Display;
use rand::Rng;
use std::time::SystemTime;

#[derive(Debug)]
enum Opcode {
    ClearScreen,                    // 00E0
    Return,                         // 00EE
    Jump(u16),                      // 1nnn
    Call(u16),                      // 2nnn
    Skip(u16,u8),                   // 3xkk
    SkipNotEqual(u16,u8),           // 4xkk
    SkipRegEqual(u16,u16),          // 5xy0
    Set(u16,u8),                    // 6xkk
    IncrementReg(u16,u8),           // 7xkk
    
    CopyReg(u16,u16),               // 8xy0
    BitwiseOr(u16,u16),             // 8xy1
    BitwiseAnd(u16,u16),            // 8xy2
    BitwiseXor(u16,u16),            // 8xy3
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

pub struct Emulator {
    v: [u8; 16],
    memory: Vec<u8>,
    pc: u16,
    i: u16,
    stack: Vec<u16>,
    sp: u16,
    display: Display,
    sound_timer: u8,
    delay_timer: u8,
    draw_flag: bool,
    frequency: u8,
    executed: Vec<Opcode>
}

impl Emulator {
    pub fn new() -> Self {

        let mut emulator = Emulator {
            v: [0; 16],
            memory: vec![0; 4096],
            pc: 0,
            i: 0,
            stack: Vec::new(),
            sp: 0,
            display: Display::new(),
            sound_timer: 0,
            delay_timer: 0,
            draw_flag: false,
            frequency: 60,
            executed: Vec::new()
        };

        let fontset = Emulator::get_fontset();

        emulator.memory[0x50..(0x50+fontset.len())].clone_from_slice(&fontset);

        //println!("{:02x?}", emulator.memory);

        emulator
    }

    pub fn load(&mut self, prog: Vec<u8>) {
        self.memory[0x200..(0x200+prog.len())].clone_from_slice(&prog[0..prog.len()]);
        self.pc = 0x200;
    }

    
    fn fetch(&mut self) -> u16 {
        let opcode1 = self.memory[(self.pc as usize)] as u16;
        self.pc += 1;
        let opcode2 = self.memory[(self.pc as usize)] as u16;
        self.pc += 1;
        opcode1 << 8 | opcode2
    }
    
    fn decode(instruction: u16) -> Opcode {
        if instruction == 238 {
            let x = 3;
        }
        match instruction {
            0x00e0 => Opcode::ClearScreen,
            0x00ee => Opcode::Return,
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
            0x8000..=0x8fff => {
                let reg1 = (instruction & 0x0f00) >> 8;    
                let reg2 = (instruction & 0x00f0) >> 4;
                let last_digit = instruction & 0x000f;
                match last_digit {
                    0x0 => Opcode::CopyReg(reg1, reg2),
                    0x1 => Opcode::BitwiseOr(reg1, reg2),
                    0x2 => Opcode::BitwiseAnd(reg1, reg2),
                    0x3 => Opcode::BitwiseXor(reg1, reg2),
                    0x4 => Opcode::AddReg(reg1, reg2),
                    0x5 => Opcode::SubtractReg(reg1, reg2),
                    0x6 => Opcode::BitwiseRight(reg1, reg2),               
                    0x7 => Opcode::NegativeSubtractReg(reg1, reg2),
                    0xe => Opcode::BitwiseLeft(reg1, reg2),                        
                    _ => panic!("Oh no!")
                }
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
            0xf000..=0xffff => {
                let reg = (instruction & 0x0f00) >> 8;
                let command_digits = instruction & 0x00ff;
                match command_digits {
                    0x07 => Opcode::CopyDelayToReg((reg) as u8),
                    0x0a => Opcode::WaitForKeyPress(reg),
                    0x15 => Opcode::SetDelayFromReg(reg),
                    0x18 => Opcode::SetSoundFromReg(reg),
                    0x1e => Opcode::AddI(reg),
                    0x29 => Opcode::SetIToFontDigit(reg),
                    0x33 => Opcode::BinaryCodeI(reg),
                    0x55 => Opcode::CopyRegistersToI(reg),
                    0x65 => Opcode::CopyIToRegisters(reg),
                    _ => panic!("Oh no!")
                }
            },
            0xe09e..=0xef9e => Opcode::SkipKeyPressed((instruction & 0x0f00) >> 8),
            0xe0a1..=0xefa1 => Opcode::SkipKeyNotPressed((instruction & 0x0f00) >> 8),
            _ => panic!("Oh no!")
        }
    }
    
    fn execute(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::ClearScreen => {
                self.display.clear();
                self.draw_flag = true;
            },
            Opcode::Return => {
                self.pc = self.pop_stack();
            },
            Opcode::Jump(addr) => {
                self.pc = addr;
            },
            Opcode::Call(addr) => {
                self.push_stack(self.pc);
                self.pc = addr;
            },
            Opcode::Skip(reg,val) => {
                if val == self.v[reg as usize] {
                    self.pc += 2;
                }
            },
            Opcode::SkipNotEqual(reg,val) => {
                if val != self.v[reg as usize] {
                    self.pc += 2;
                }
            },
            Opcode::SkipRegEqual(reg1,reg2) => {
                if self.v[reg1 as usize] == self.v[reg2 as usize] {
                    self.pc += 2;
                }
            },                     
            Opcode::Set(v, n) => self.v[v as usize] = n,
            Opcode::IncrementReg(v, n) => self.v[v as usize] = self.v[v as usize].wrapping_add(n),
            Opcode::CopyReg(reg1, reg2) => self.v[reg1 as usize] = self.v[reg2 as usize],
            Opcode::BitwiseOr(reg1, reg2) => self.v[reg1 as usize] |= self.v[reg2 as usize],
            Opcode::BitwiseAnd(reg1, reg2) => self.v[reg1 as usize] &= self.v[reg2 as usize],
            Opcode::BitwiseXor(reg1, reg2) => self.v[reg1 as usize] ^= self.v[reg2 as usize],
            Opcode::AddReg(reg1, reg2) => {
                let sum = self.v[reg1 as usize] as usize + self.v[reg2 as usize] as usize;
                if sum > 255 {
                    self.v[15] = 1;
                    self.v[reg1 as usize] = (sum % 0x255) as u8;
                }
                else {
                    self.v[reg1 as usize] = sum as u8;
                }
            },
            Opcode::SubtractReg(reg1, reg2) => {
                let v1 = self.v[reg1 as usize] as i16;
                let v2 = self.v[reg2 as usize] as i16;
                self.v[reg1 as usize] = (v1-v2) as u8;
                if v1 >= v2 {
                    self.v[15] = 1;
                }
                else {
                    self.v[15] = 0;
                }
            },
            Opcode::BitwiseRight(reg1, _) => {
                let least_bit = (self.v[reg1 as usize] & 1) as u8;
                self.v[15] = least_bit;
                self.v[reg1 as usize] >>= 1;
            },
            Opcode::NegativeSubtractReg(reg1, reg2) => {
                let v1 = self.v[reg1 as usize];
                let v2 = self.v[reg2 as usize];
                if v2 > v1 {
                    self.v[reg1 as usize] = v2 - v1;
                    self.v[15] = 1;
                }
                else {
                    self.v[15] = 0;
                }                
            },
            Opcode::BitwiseLeft(reg1, _) => {
                let most_significant_bit = (self.v[reg1 as usize] & 0b10000000) >> 7;
                self.v[15] = most_significant_bit;
                self.v[reg1 as usize] <<= 1;                
            },
            Opcode::SkipRegNotEqual(reg1,reg2) => {
                if self.v[reg1 as usize] != self.v[reg2 as usize] {
                    self.pc += 2;
                }
            },              
            Opcode::SetI(i) => self.i = i,
            Opcode::JumpOffset(offset) => {
                self.pc = self.v[0] as u16 + offset;
            },
            Opcode::RandomAnd(reg, val) => {
                let rando: u8 = rand::thread_rng().gen();
                self.v[reg as usize] = rando & val;
            },
            Opcode::Draw(x, y, rows) => {
                let vx = self.v[x as usize] % 64;
                let vy = self.v[y as usize] % 32;
                //println!("row count = {}", rows);
                //let ourrange = &self.memory[self.I as usize..=(self.I as usize+rows as usize)];
                //println!("I-slice = {:04x?}", ourrange);                
                for i in 0..rows {
                    let row_byte = self.memory[self.i as usize +i as usize];
                    let x_coord = (vx % 64) as usize;
                    let y_coord = ((vy + i) % 32) as usize;                        
                    let collision = self.display.write_row_buffer(x_coord, y_coord, row_byte);
                    if collision {
                        self.v[15] = 1;
                    }
                }

                self.draw_flag = true;
            },
            Opcode::SkipKeyPressed(reg1) => {
                let val = self.v[reg1 as usize];
                if let Some(key) = Keypad::pressed() {
                    if key == val as u32 {
                        self.pc += 2;
                    }
                }             
            },
            Opcode::SkipKeyNotPressed(reg1) => {
                let val = self.v[reg1 as usize];
                if let Some(key) = Keypad::pressed() {
                    if key != val as u32 {
                        self.pc += 2;
                    }
                }
                else {
                    self.pc += 2;
                }                   
            },
            Opcode::CopyDelayToReg(reg1) => self.v[reg1 as usize] = self.delay_timer,
            Opcode::WaitForKeyPress(reg1) => {
                loop {
                    if let Some(key) = Keypad::pressed() {
                        self.v[reg1 as usize] = key as u8;
                        break;
                    }                         
                }
            },
            Opcode::SetDelayFromReg(reg1) => self.delay_timer = self.v[reg1 as usize],
            Opcode::SetSoundFromReg(reg1) => self.sound_timer = self.v[reg1 as usize],
            Opcode::AddI(val) => self.i += val,
            Opcode::SetIToFontDigit(reg) => {
                let val = self.v[reg as usize];
                self.i = 0x50 + val as u16;
            },
            Opcode::BinaryCodeI(reg) => {
                let val_string = format!("{:03}",self.v[reg as usize]);
                let digits: Vec<u8> = val_string
                    .chars()
                    .map(|d| d.to_digit(10).unwrap() as u8)
                    .collect();

                self.memory[self.i as usize] = digits[0];
                self.memory[(self.i+1) as usize] = digits[1];
                self.memory[(self.i+2) as usize] = digits[2];

            },
            Opcode::CopyRegistersToI(n) => {
                for i in 0..=n {
                    self.memory[(self.i+i) as usize] = self.v[i as usize];
                }
            },
            Opcode::CopyIToRegisters(n) => {
                for i in 0..=n {
                    self.v[i as usize] = self.memory[(self.i+i) as usize];
                }
            }
        }
        self.executed.push(opcode);
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

    pub fn run(&mut self) {           
        let mut last_cycle_time =SystemTime::now();
        let max_duration = (1000_f64)/(self.frequency as f64);

        loop {
            let opcode = self.fetch();
            let instruction = Self::decode(opcode);
            if opcode == 0x0 { break; }
            //println!("{:04x}: Opcode =      {:04x?}", self.pc-2, opcode);
            //println!("{:04x}: Instruction = {:?}", self.pc-2, instruction);

            self.execute(instruction);
            if self.draw_flag {
                self.draw();
            }
            
            let time_elapsed = SystemTime::now().duration_since(last_cycle_time).unwrap().as_millis();
            if time_elapsed >= max_duration as u128 {
                if self.delay_timer > 0 {
                    self.delay_timer-= 1;
                }
                last_cycle_time = SystemTime::now();
            }
        }    
    }

    fn draw(&mut self) {
        print!("{}[2J", 27 as char);
        print!("╔");
        let width = self.display.pixels[0].len();
        print!("{:═<1$}", "", width);
        // print ═ n times, where n is display width -2
        println!("╗");
        for row in self.display.pixels {
            print!("║");
            for pixel in row {
                if pixel {
                    print!("▉"); //("█") // ▀
                }
                else {
                    print!(" ");
                }
            }
            println!("║");
        }
        print!("╚");
        print!("{:═<1$}", "", width);
        println!("╝");
        println!();
        print!("DT");
        println!();
        println!("Last executed: {:?}", self.executed.last().unwrap());
        for (i,val) in self.v.iter().enumerate() {
            print!("V{} {:x}  ", i, val);
        }
        println!();
        self.draw_flag = false;
    }

}
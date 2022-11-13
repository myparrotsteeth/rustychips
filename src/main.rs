extern crate rand;

use std::fs;

mod emulator;

use crate::emulator::emulator::Emulator;

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

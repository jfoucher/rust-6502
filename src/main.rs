use std::env;

use std::fs;

fn main() {

    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    // read the whole file
    let buffer = fs::read(filename).expect("could not read file");

    let pc :i16 = 0;
    let ram = [i8; 0x8000];
    let clock: i16 = 0;
}
use std::env;

use std::fs;

fn main() {

    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    // read the whole file
    let addresses = fs::read(filename).expect("could not read file");

    let ram = &addresses[..0x8000];
    let rom = &addresses[0x8000..0xffff];

    println!("{:x?}", rom[0x7ffe]);
    let clock: u64 = 0;
    let pc :u16 = 0;

    loop {
        (pc, clock) = run_instruction(addresses[pc as usize], pc, clock);

        if (pc > 5) {
            break;
        }
    }
}

fn run_instruction(inst: u8, pc: u16, clock: u64) -> (u16, u64) {
    println!("Running instruction {:x?}", inst);
    match inst {
        0x69 => adc(pc, clock),
        _ => nop(pc, clock),
    };

    return (pc, clock);
}
fn adc(pc: u16, clock: u64) -> (u16, u64) {
    pc += 1;
    clock += 1;
    return (pc, clock);
}

fn nop(pc: u16, clock: u64) -> (u16, u64) {
    pc += 1;
    clock += 1;
    return (pc, clock);
}
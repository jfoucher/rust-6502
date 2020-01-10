use std::env;
use std::convert::TryInto;
use std::fs;
use std::{thread, time};



#[derive(Debug)]
struct Processor {
    
    flags: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    pc: u16,
    sp: u8,
}


fn main() {

    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    // read the whole file
    let mut addresses: Vec<u8>;
    addresses = fs::read(filename).expect("could not read file");

    let mut clock: u64 = 0;

    let mut proc = Processor {
        flags: 0,
        acc: 0,
        rx: 0,
        ry: 0,
        pc: 0x400,
        sp: 0
    };

    let millis = time::Duration::from_millis(100);



    let mut i = 0;
    loop {
        let (_addr, _proc) = run_instruction(addresses, proc);
        proc = _proc;
        addresses = _addr;
        println!("proc after run_instruction {:?}", proc);

        i += 1;
        if i > 350 {
            break;
        }
        //thread::sleep(millis);
    }
}

fn run_instruction(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let inst = addresses[proc.pc as usize];

    match inst {
        0x10 => {
            println!("Running instruction bpl : {:x?}", inst);
            return bpl(addresses, proc);
        },
        0x18 => {
            println!("Running instruction clc : {:x?}", inst);
            return clc(addresses, proc);
        },
        0x49 => {
            println!("Running instruction eor : {:x?}", inst);
            return eor(addresses, proc);
        },
        0x4c => {
            println!("Running instruction jmp : {:x?}", inst);
            return jmp(addresses, proc);
        },
        0x69 => {
            println!("Running instruction adc : {:x?}", inst);
            return adc(addresses, proc);
        },
        0x88 => {
            println!("Running instruction dey : {:x?}", inst);
            return dey(addresses, proc);
        },
        0x8d => {
            println!("Running instruction sta : {:x?}", inst);
            return sta(addresses, proc);
        },

        0x98 => {
            println!("Running instruction tya : {:x?}", inst);
            return tya(addresses, proc);
        },
        0x9a => {
            println!("Running instruction txs : {:x?}", inst);
            return txs(addresses, proc);
        },
        0xa0 => {
            println!("Running instruction ldy : {:x?}", inst);
            return ldy(addresses, proc);
        },
        0xa2 => {
            println!("Running instruction ldx : {:x?}", inst);
            return ldx(addresses, proc);
        },

        0xaa => {
            println!("Running instruction tax : {:x?}", inst);
            return tax(addresses, proc);
        },
        0xa9 | 0xad => {
            println!("Running instruction lda : {:x?}", inst);
            return lda(addresses, proc, inst);
        },
        0xc9 => {
            println!("Running instruction cmp : {:x?}", inst);
            return cmp(addresses, proc);
        },
        0xca => {
            println!("Running instruction dex : {:x?}", inst);
            return dex(addresses, proc);
        },
        0xd0 => {
            println!("Running instruction bne : {:x?}", inst);
            return bne(addresses, proc);
        },
        0xd8 => {
            println!("Running instruction cld : {:x?}", inst);
            return cld(addresses, proc);
        },
        0xf0 => {
            println!("Running instruction beq : {:x?}", inst);
            return beq(addresses, proc);
        },
        _ => {
            println!("Running instruction nop : {:x?}", inst);
            return nop(addresses, proc);
        },
    };
}
fn adc(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let mut acc = proc.acc;
    let val = addresses[(proc.pc + 1) as usize];
    acc += val;
    return (addresses.to_vec(), Processor { pc: proc.pc + 2, acc, ..proc });
}

fn cld(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags: proc.flags & 0x7, ..proc });
}

fn txs(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, sp: proc.rx, ..proc });
}

fn tya(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, acc: proc.ry, ..proc });
}

fn clc(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = proc.flags & 0xFE;
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, ..proc });
}

fn tax(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, rx: proc.acc, ..proc });
}

fn eor(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let val = addresses[(proc.pc + 1) as usize];
    let mut acc = proc.acc;
    println!("EOR {:x?} {:x?}", val, acc);
    acc ^= val;
    return (addresses.to_vec(), Processor { pc: proc.pc + 2, acc, ..proc });
}

fn ldx(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let x = addresses[(proc.pc + 1) as usize];
    let flags = set_flags(proc.flags, x);
    return (addresses.to_vec(), Processor { rx: x, flags, pc: proc.pc + 2, ..proc });
}

fn ldy(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let y = addresses[(proc.pc + 1) as usize];
    let flags = set_flags(proc.flags, y);
    return (addresses.to_vec(), Processor { ry: y, flags, pc: proc.pc + 2, ..proc });
}

fn lda(addresses: Vec<u8>, proc: Processor, inst:u8) -> (Vec<u8>, Processor) {
    let mut acc: u8 = addresses[(proc.pc + 1) as usize];
    let mut pc = proc.pc + 2;
    if inst == 0xad {
        //Absolute adressing

        let addr = get_word(&addresses, proc.pc + 1);
        println!("inst is absolute addr {:x?}", addr);
        acc = addresses[addr as usize];
        pc = proc.pc + 3;
    }
    let flags = set_flags(proc.flags, acc);
    return (addresses.to_vec(), Processor { acc, pc, flags, ..proc });
}

fn dex(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let rx = proc.rx - 1;
    let flags = set_flags(proc.flags, rx);
    return (addresses.to_vec(), Processor { rx, flags, pc: proc.pc + 1, ..proc });
}

fn dey(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let ry = proc.ry - 1;
    let flags = set_flags(proc.flags, ry);

    return (addresses.to_vec(), Processor { ry, flags, pc: proc.pc + 1, ..proc });
}

fn cmp(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let acc = proc.acc;
    let value = addresses[(proc.pc + 1) as usize];
    let result: u8 = acc.wrapping_sub(value);
    let mut flags = proc.flags;
    if (acc > value) {
        flags |= 1;
    }
    flags = set_flags(flags, result as u8);

    return (addresses.to_vec(), Processor { flags, pc: proc.pc + 2, ..proc });
}

fn sta(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let addr = get_word(&addresses, proc.pc + 1);
    println!("sta addr 0x{:x?}", addr);
    let mut _addr = addresses.to_vec();
    _addr[addr as usize] = proc.acc;
    println!("before store 0x{:x?}", _addr[addr as usize]);
    println!("to store 0x{:x?}", proc.acc);
    println!("sta value 0x{:x?}", _addr[addr as usize]);
    return (_addr.to_vec(), Processor { pc: proc.pc + 3, ..proc });
}

fn jmp(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let addr = get_word(&addresses, proc.pc + 1);
    println!("Jumping to 0x{:x?}", addr);
    return (addresses.to_vec(), Processor { pc: addr, ..proc });
}

fn bne(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 1) & 1 == 0;
    let mut new_addr :u16;
    new_addr = proc.pc + 2;
    if (should_jump) {
        let rel_address = offset as i8;
        println!("Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
    }

    println!("Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, ..proc });
}


fn beq(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 1) & 1 == 1;
    let mut new_addr :u16;
    new_addr = proc.pc + 2;
    if (should_jump) {
        let rel_address = offset as i8;
        println!("Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
    }

    println!("Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, ..proc });
}

fn bpl(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 7) & 1 == 0;
    let mut new_addr :u16;
    new_addr = proc.pc + 2;
    if (should_jump) {
        let rel_address = offset as i8;
        println!("BPL Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
    }

    println!("BPL Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, ..proc });
}

fn nop(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, ..proc });
}

fn set_flags(flags:u8, val:u8) -> u8 {
    let mut _flags = flags;
    if val == 0 {
        //Set zero flag
        _flags |= 0b10;
    } else {
        _flags &= 0b11111101;
    }
    if (val >> 7 == 1) {
        _flags |= 0b10000000;
    }
    // println!("Setting flags to {:#b}", _flags);
    return _flags;
}

fn get_word(data: &Vec<u8>, address: u16) -> u16 {
    let low_byte :u16 = data[(address) as usize].into();
    let high_byte :u16 = data[(address + 1) as usize].into();
    return low_byte + (high_byte << 8);
}
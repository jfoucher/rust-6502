use std::env;
use std::convert::TryInto;
use std::fs;

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

    let mut i = 0;
    loop {
        let (_addr, _proc) = run_instruction(addresses, proc);
        proc = _proc;
        addresses = _addr;
        println!("proc after run_instruction {:?}", proc);

        i += 1;
        if i > 20 {
            break;
        }
    }
}

fn run_instruction(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let inst = addresses[proc.pc as usize];

    match inst {
        0x4c => {
            println!("Running instruction jmp : {:x?}", inst);
            return jmp(addresses, proc);
        },
        0x69 => {
            println!("Running instruction adc : {:x?}", inst);
            return adc(addresses, proc);
        },
        0x8d => {
            println!("Running instruction sta : {:x?}", inst);
            return sta(addresses, proc);
        },
        0x9a => {
            println!("Running instruction txs : {:x?}", inst);
            return txs(addresses, proc);
        },
        0xa2 => {
            println!("Running instruction ldx : {:x?}", inst);
            return ldx(addresses, proc);
        },
        0xa9 => {
            println!("Running instruction lda : {:x?}", inst);
            return lda(addresses, proc);
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
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, ..proc });
}

fn cld(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags: proc.flags & 0x7, ..proc });
}

fn txs(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, sp: proc.rx, ..proc });
}

fn ldx(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let x = addresses[(proc.pc + 1) as usize];
    let mut flags = proc.flags;
    if x == 0 {
        //Set zero flag
        flags = flags | 0b10;
    } else {
        flags = flags & 0b11111101;
    }
    if (x >> 7 == 1) {
        flags = flags | 0b01000000;
    }
    return (addresses.to_vec(), Processor { rx: x, flags, pc: proc.pc + 2, ..proc });
}

fn ldy(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let x = addresses[(proc.pc + 1) as usize];
    let mut flags = proc.flags;
    if x == 0 {
        //Set zero flag
        flags = flags | 0b10;
    } else {
        flags = flags & 0b11111101;
    }
    if (x >> 7 == 1) {
        flags = flags | 0b01000000;
    }
    return (addresses.to_vec(), Processor { rx: x, flags, pc: proc.pc + 2, ..proc });
}

fn lda(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { acc: addresses[(proc.pc + 1) as usize], pc: proc.pc + 2, ..proc });
}
fn dex(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let rx = proc.rx - 1;
    let mut flags = proc.flags;
    if rx == 0 {
        //Set zero flag
        flags = flags | 0b10;
    } else {
        flags = flags & 0b11111101;
    }
    return (addresses.to_vec(), Processor { rx, flags, pc: proc.pc + 1, ..proc });
}
fn sta(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let address_low_byte :u16 = addresses[(proc.pc + 1) as usize].into();
    let address_high_byte :u16 = addresses[(proc.pc + 2) as usize].into();
    let addr :u16 = address_low_byte + (address_high_byte << 8);
    println!("sta addr 0x{:x?}", addr);
    let mut _addr = addresses.to_vec();
    _addr[addr as usize] = proc.acc;
    println!("before store 0x{:x?}", _addr[addr as usize]);
    println!("to store 0x{:x?}", proc.acc);
    println!("sta value 0x{:x?}", _addr[addr as usize]);
    return (_addr.to_vec(), Processor { pc: proc.pc + 3, ..proc });
}

fn jmp(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let address_low_byte :u16 = addresses[(proc.pc + 1) as usize].into();
    let address_high_byte :u16 = addresses[(proc.pc + 2) as usize].into();
    let addr :u16 = address_low_byte + (address_high_byte << 8);
    println!("Jumping to 0x{:x?}", addr);
    return (addresses.to_vec(), Processor { pc: addr, ..proc });
}

fn bne(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    
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

fn nop(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, ..proc });
}
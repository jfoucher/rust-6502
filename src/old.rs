use std::env;
//use std::convert::TryInto;
use std::fs;
use std::time;
use std::thread;
use std::error::Error;
use cursive::align::HAlign;
use cursive::traits::*;
use cursive::views::{Dialog, DummyView, LinearLayout, TextView, OnEventView, Canvas, IdView};
use cursive::Cursive;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;

#[derive(Clone, Debug)]
struct ModelData {
    /// The offset will be controlled by the UI and used in the server
    processor: Processor,
    /// Logs will be filled by the server and displayed on the UI
    data: Vec<u8>,
    // A callback sink is used to control the UI from the server
    // (eg. force refresh, error popups)
    //cb_sink: cursive::CbSink,
}

type Model = Arc<Mutex<ModelData>>;


#[derive(Clone, Debug)]
struct Processor {
    
    flags: u8,
    acc: u8,
    rx: u8,
    ry: u8,
    pc: u16,
    sp: u8,
    info: String,
    clock: u64,
    speed: u64,
}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
  let strs: Vec<String> = bytes.iter()
                               .map(|b| format!("{:02X}", b))
                               .collect();
  strs.connect(" ")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let filename = &args[1];

    // read the whole file
    let mut addresses: Vec<u8>;
    addresses = fs::read(filename).expect("could not read file");

    let mut proc = Processor {
        flags: 0,
        acc: 0,
        rx: 0,
        ry: 0,
        pc: 0x400,
        sp: 0,
        info: "".to_string(),
        clock: 0,
        speed: 100,
    };

    let mut siv = Cursive::default();

    siv.add_global_callback('q', Cursive::quit);

    let mut model = ModelData {
        processor: proc,
        data: addresses,
        cb_sink: siv.cb_sink().clone(),
    };

    //let (sender, receiver): (mpsc::Sender<ModelData>, mpsc::Receiver<ModelData>) = mpsc::channel::<ModelData>();
    //let (timer_sender, timer_receiver): (mpsc::Sender<u32>, mpsc::Receiver<u32>) = mpsc::channel::<u32>();
    //let (sender, receiver): (mpsc::Sender<ModelData>, mpsc::Receiver<ModelData>) = mpsc::channel();

    // let child = thread::spawn(move || {
    //     simulate(model, sender);
    // });

    // let ts1 = timer_sender.clone();
    // let ts2 = timer_sender.clone();

    siv.screen_mut().add_layer(
        Dialog::around(build_ui(model.clone()))
            .title("6502 Simulator")
            .button("Quit", |s| {
                std::process::abort();
                std::process::exit(0);
            })
            // .button("Slower", move |s| {
            //     ts1.send(1);
            // })
            // .button("Faster", move |s| {
            //     ts2.send(0);
            // })
            .full_screen()
            ,
    );

    let mut i = 0;
    let mut millis = 100;
    loop {
        siv.step();
        let (_addr, _proc) = run_instruction(model.data, model.processor);
        //proc = _proc;
        //addresses = _addr;
        // if let Ok(change_timer) = timer_receiver.try_recv() {
        //     if (change_timer == 1) {
        //         millis *= 2;
        //     } else {
        //         millis /= 2;
        //     }
        // }

        let mut _pr = _proc.clone();
        _pr.speed = millis;
        model.data = _addr;
        model.processor = _pr;
        //// println!("{}", text.to_string());

        
        i += 1;
        if i > 5000 {
            break;
        }
        siv.call_on_id("pc", |view: &mut TextView| {
            view.set_content(format!("{} ({:#x})", model.processor.pc, model.processor.pc));
        });
        siv.call_on_id("flags", |view: &mut TextView| {
            view.set_content(format!("{:b}", model.processor.flags));
        });

        siv.call_on_id("clock", |view: &mut TextView| {
            view.set_content(format!("{}", model.processor.clock));
        });
        siv.call_on_id("acc", |view: &mut TextView| {
            view.set_content(format!("{} ({:#x})", model.processor.acc, model.processor.acc));
        });
        siv.call_on_id("rx", |view: &mut TextView| {
            view.set_content(format!("{} ({:#x})", model.processor.rx, model.processor.rx));
        });
        siv.call_on_id("ry", |view: &mut TextView| {
            view.set_content(format!("{} ({:#x})", model.processor.ry, model.processor.ry));
        });
        siv.call_on_id("speed", |view: &mut TextView| {
            view.set_content(format!("{}", model.processor.speed));
        });
        siv.call_on_id("info", |view: &mut TextView| {
            //let content = view.get_content().source();
            let fmt = format!("{}\n{}", model.processor.info, view.get_content().source());
            view.set_content(fmt);
        });
        siv.call_on_id("mem", |view: &mut TextView| {
            let btm :u16 = if model.processor.pc > 256 { (model.processor.pc - 255) }else {0};
            let top :u16 = if (model.processor.pc < 0xffff - 256) { model.processor.pc + 256} else { 0xffff };
            let var = to_hex_string(model.data[btm as usize ..=top as  usize].to_vec());
            view.set_content(var);
        });
            
        
        siv.refresh();
        //// println!("output: {:?}", output); 
    }

    //siv.run();
    // Build the UI from the model
    
    //let res = child.join();
    
}

fn build_ui(model: ModelData) -> impl cursive::view::View {
    // Build the UI in 3 parts, stacked together in a LinearLayout.

    LinearLayout::horizontal()
    .child(LinearLayout::vertical()
        .child(build_mem_viewer(model.clone())).fixed_width(16*3)
    )
    .child(DummyView.fixed_width(50))
    .child(
        LinearLayout::vertical()
            .child(build_proc_viewer(model.clone()))
            .child(DummyView.fixed_height(1))
            .child(build_debug_viewer(model.clone()))
    )
}

fn build_debug_viewer(model: ModelData) -> impl cursive::view::View {
    Dialog::around(
        LinearLayout::vertical()
        .child(
            TextView::new(format!("{}", model.processor.info)).with_id("info")
        )
        
        .scrollable()
        .scroll_x(true)
        .scroll_y(true)
    )
    .title("Debug info")
    .fixed_size((20, 20))
    
}

fn build_proc_viewer(model: ModelData) -> impl cursive::view::View  {
    Dialog::around(
        LinearLayout::horizontal()
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("PC")
                )
                .child(
                    TextView::new(format!("{} ({:#x})", model.processor.pc, model.processor.pc)).with_id("pc")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("Flags")
                )
                .child(
                    TextView::new(format!("{:b}", model.processor.flags)).with_id("flags")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("Acc")
                )
                .child(
                    TextView::new(format!("{:b}", model.processor.acc)).with_id("acc")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("RX")
                )
                .child(
                    TextView::new(format!("{:b}", model.processor.rx)).with_id("rx")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("RY")
                )
                .child(
                    TextView::new(format!("{:b}", model.processor.ry)).with_id("ry")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("CLOCK")
                )
                .child(
                    TextView::new(format!("{}", model.processor.clock)).with_id("clock")
                )
        )
        .child(DummyView.fixed_width(1))
        .child(
            LinearLayout::vertical()
                .child(
                    TextView::new("Speed")
                )
                .child(
                    TextView::new(format!("{}", model.processor.speed)).with_id("speed")
                )
        )
    )
    .title("Processor status")
}
fn build_mem_viewer(model: ModelData) -> impl cursive::view::View {
    let btm :u16 = if model.processor.pc >= 512 { (model.processor.pc - 512) }else {0};
    let top :u16 = if (model.processor.pc < 0xffff - 512) { model.processor.pc + 512} else { 0xffff };
    let var = to_hex_string(model.data[btm as usize ..=top as  usize].to_vec());
    TextView::new(var).with_id("mem")
}

fn run_instruction(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let inst = addresses[proc.pc as usize];

    match inst {
        0x10 => {
            //// println!("Running instruction bpl : {:x?}", inst);
            return bpl(addresses, proc);
        },
        0x18 => {
            //// println!("Running instruction clc : {:x?}", inst);
            return clc(addresses, proc);
        },
        0x49 => {
            //// println!("Running instruction eor : {:x?}", inst);
            return eor(addresses, proc);
        },
        0x4c => {
            //// println!("Running instruction jmp : {:x?}", inst);
            return jmp(addresses, proc);
        },
        0x69 => {
            //// println!("Running instruction adc : {:x?}", inst);
            return adc(addresses, proc);
        },
        0x88 => {
            //// println!("Running instruction dey : {:x?}", inst);
            return dey(addresses, proc);
        },
        0x8d => {
            //// println!("Running instruction sta : {:x?}", inst);
            return sta(addresses, proc);
        },

        0x98 => {
            //// println!("Running instruction tya : {:x?}", inst);
            return tya(addresses, proc);
        },
        0x9a => {
            //// println!("Running instruction txs : {:x?}", inst);
            return txs(addresses, proc);
        },
        0xa0 => {
            //// println!("Running instruction ldy : {:x?}", inst);
            return ldy(addresses, proc);
        },
        0xa2 => {
            //// println!("Running instruction ldx : {:x?}", inst);
            return ldx(addresses, proc);
        },

        0xaa => {
            //// println!("Running instruction tax : {:x?}", inst);
            return tax(addresses, proc);
        },
        0xa9 | 0xad => {
            //// println!("Running instruction lda : {:x?}", inst);
            return lda(addresses, proc, inst);
        },
        0xc9 => {
            //// println!("Running instruction cmp : {:x?}", inst);
            return cmp(addresses, proc);
        },
        0xca => {
            //// println!("Running instruction dex : {:x?}", inst);
            return dex(addresses, proc);
        },
        0xd0 => {
            //// println!("Running instruction bne : {:x?}", inst);
            return bne(addresses, proc);
        },
        0xd8 => {
            //// println!("Running instruction cld : {:x?}", inst);
            return cld(addresses, proc);
        },
        0xf0 => {
            //// println!("Running instruction beq : {:x?}", inst);
            return beq(addresses, proc);
        },
        _ => {
            //// println!("Running instruction nop : {:x?}", inst);
            return nop(addresses, proc);
        },
    };
}
fn adc(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let mut acc = proc.acc;
    let val = addresses[(proc.pc + 1) as usize];
    acc += val;
    let info = format!("Running instruction adc: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 2, acc, info, clock: proc.clock + 1, ..proc });
}

fn cld(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let info = format!("Running instruction adc: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags: proc.flags & 0x7, info, clock: proc.clock + 1, ..proc });
}

fn txs(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    let info = format!("Running instruction txs: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, sp: proc.rx, info, clock: proc.clock + 1, ..proc });
}

fn tya(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    let info = format!("Running instruction tya: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, info, acc: proc.ry, clock: proc.clock + 1, ..proc });
}

fn clc(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = proc.flags & 0xFE;
    let info = format!("Running instruction clc: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, info, flags, clock: proc.clock + 1, ..proc });
}

fn tax(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let flags = set_flags(proc.flags, proc.rx);
    let info = format!("Running instruction tax: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, flags, rx: proc.acc, info, clock: proc.clock + 1, ..proc });
}

fn eor(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let val = addresses[(proc.pc + 1) as usize];
    let mut acc = proc.acc;
    //// println!("EOR {:x?} {:x?}", val, acc);
    let info = format!("Running instruction eor: {:#x}", addresses[(proc.pc) as usize]);
    acc ^= val;
    return (addresses.to_vec(), Processor { pc: proc.pc + 2, acc, info, clock: proc.clock + 1, ..proc });
}

fn ldx(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let x = addresses[(proc.pc + 1) as usize];
    let flags = set_flags(proc.flags, x);
    let info = format!("Running instruction ldx: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { rx: x, flags, pc: proc.pc + 2, info, clock: proc.clock + 1, ..proc });
}

fn ldy(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let y = addresses[(proc.pc + 1) as usize];
    let flags = set_flags(proc.flags, y);
    let info = format!("Running instruction ldy: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { ry: y, flags, pc: proc.pc + 2, info, clock: proc.clock + 1, ..proc });
}

fn lda(addresses: Vec<u8>, proc: Processor, inst:u8) -> (Vec<u8>, Processor) {
    let mut acc: u8 = addresses[(proc.pc + 1) as usize];
    let mut pc = proc.pc + 2;
    let mut info = format!("Running instruction lda: {:#x}", inst);
    if inst == 0xad {
        //Absolute adressing

        let addr = get_word(&addresses, proc.pc + 1);
        //// println!("inst is absolute addr {:x?}", addr);
        acc = addresses[addr as usize];
        pc = proc.pc + 3;
        info = format!("Running instruction lda absolute: {:#x}", inst);
    }
    let flags = set_flags(proc.flags, acc);
    return (addresses.to_vec(), Processor { acc, pc, flags, info, clock: proc.clock + 1, ..proc });
}

fn dex(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let rx = proc.rx - 1;
    let flags = set_flags(proc.flags, rx);
    let info = format!("Running instruction dex: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { rx, flags, pc: proc.pc + 1, info, clock: proc.clock + 1, ..proc });
}

fn dey(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let ry = proc.ry - 1;
    let flags = set_flags(proc.flags, ry);
    let info = format!("Running instruction dey: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { ry, flags, pc: proc.pc + 1, info, clock: proc.clock + 1, ..proc });
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
    let info = format!("Running instruction cmp: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { flags, pc: proc.pc + 2, info, clock: proc.clock + 1, ..proc });
}

fn sta(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let addr = get_word(&addresses, proc.pc + 1);
   // // println!("sta addr 0x{:x?}", addr);
    let mut _addr = addresses.to_vec();
    _addr[addr as usize] = proc.acc;
    //// println!("before store 0x{:x?}", _addr[addr as usize]);
    //// println!("to store 0x{:x?}", proc.acc);
    //// println!("sta value 0x{:x?}", _addr[addr as usize]);
    let info = format!("Running instruction sta: {:#x}", addresses[(proc.pc) as usize]);
    return (_addr.to_vec(), Processor { pc: proc.pc + 3, info, clock: proc.clock + 1, ..proc });
}

fn jmp(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let addr = get_word(&addresses, proc.pc + 1);
    //// println!("Jumping to 0x{:x?}", addr);
    let info = format!("Running instruction jmp: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: addr, info, clock: proc.clock + 1, ..proc });
}

fn bne(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    //// println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 1) & 1 == 0;
    let mut new_addr :u16;
    new_addr = proc.pc + 2;
    let mut info = format!("Running instruction bne not jumping: {:#x}", addresses[(proc.pc) as usize]);
    if (should_jump) {
        let rel_address = offset as i8;
        // // println!("Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        info = format!("Running instruction bne {:#x} jumping to: {:#x}", addresses[(proc.pc) as usize], new_addr);
    }
    
    // // println!("Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, info, clock: proc.clock + 1, ..proc });
}


fn beq(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 1) & 1 == 1;
    let mut new_addr :u16;
    let mut info = format!("Running instruction beq not jumping: {:#x}", addresses[(proc.pc) as usize]);
    new_addr = proc.pc + 2;
    if (should_jump) {
        let rel_address = offset as i8;
        // // println!("Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        info = format!("Running instruction beq {:#x} jumping to: {:#x}", addresses[(proc.pc) as usize], new_addr);
    }

    // println!("Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, info, clock: proc.clock + 1, ..proc });
}

fn bpl(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let offset = addresses[(proc.pc + 1) as usize];
    // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
    let should_jump = (proc.flags >> 7) & 1 == 0;
    let mut new_addr :u16;
    new_addr = proc.pc + 2;
    let mut info = format!("Running instruction bpl not jumping: {:#x}", addresses[(proc.pc) as usize]);
    if (should_jump) {
        let rel_address = offset as i8;
        // println!("BPL Jumping offset {:?}", rel_address);
        new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        info = format!("Running instruction bpl {:#x} jumping to: {:#x}", addresses[(proc.pc) as usize], new_addr);
    }

    // println!("BPL Jumping to 0x{:x?}", new_addr);
    return (addresses.to_vec(), Processor { pc: new_addr, info, clock: proc.clock + 1, ..proc });
}

fn nop(addresses: Vec<u8>, proc: Processor) -> (Vec<u8>, Processor) {
    let info = format!("Running instruction nop: {:#x}", addresses[(proc.pc) as usize]);
    return (addresses.to_vec(), Processor { pc: proc.pc + 1, info, clock: proc.clock + 1, ..proc });
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
    // // println!("Setting flags to {:#b}", _flags);
    return _flags;
}

fn get_word(data: &Vec<u8>, address: u16) -> u16 {
    let low_byte :u16 = data[(address) as usize].into();
    let high_byte :u16 = data[(address + 1) as usize].into();
    return low_byte + (high_byte << 8);
}
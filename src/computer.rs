use std::sync::mpsc;
use std::time;
use std::thread;

#[derive(Clone, Debug)]
pub struct Info {
    pub msg: String,
    pub qty: u64,
}

pub enum ControllerMessage {
    ButtonPressed(String),
    UpdatedProcessorAvailable(Processor),
    UpdatedDataAvailable(Vec<u8>),
    UpdatedStackAvailable(Vec<u8>),
}

pub enum ComputerMessage {
    ButtonPressed(String),
    GetData(),
}

#[derive(Clone, Debug)]
pub struct Processor {
    pub flags: u8,
    pub acc: u8,
    pub rx: u8,
    pub ry: u8,
    pub pc: u16,
    pub sp: u8,
    pub test: u8,
    pub info: Vec<Info>,
    pub clock: u64,
}

#[derive(Debug)]
pub struct Computer {
    processor: Processor,
    paused: bool,
    step: bool,
    speed: u64,
    data: Vec<u8>,
    tx: mpsc::Sender<ControllerMessage>,
    rx: mpsc::Receiver<ComputerMessage>,
}
const FLAG_C: u8 = 1;
const FLAG_Z: u8 = 2;
const FLAG_I: u8 = 4;
const FLAG_D: u8 = 8;
const FLAG_O: u8 = 0x40;
const FLAG_N: u8 = 0x80;

#[derive(PartialEq)]
enum ADRESSING_MODE {
    IMMEDIATE = 0,
    ZERO_PAGE = 1,	
    ZERO_PAGE_X = 2,
    ABSOLUTE = 3,
    ABSOLUTE_X = 4,
    ABSOLUTE_Y = 5,
    INDIRECT_X = 6,
    INDIRECT_Y = 7,
}

impl Computer {
    pub fn new(tx: mpsc::Sender<ControllerMessage>, rx:  mpsc::Receiver<ComputerMessage>, data: Vec<u8>) -> Computer {
        let mut computer = Computer {
            data,
            tx,
            rx,
            paused: true,
            step: false,
            speed: 0,
            processor: Processor {
                flags: 0,
                acc: 0,
                rx: 0,
                ry: 0,
                /// Start at 0x400
                pc: 0x400,
                sp: 0,
                test: 0,
                info: vec![],
                clock: 0,
            }
        };
        computer
    }

    pub fn step(&mut self) -> bool {
        while let Some(message) = self.rx.try_iter().next() {
            // Handle messages arriving from the controller.
            match message {
                ComputerMessage::ButtonPressed(btn) => {
                    if btn == "faster" && self.speed > 0 {
                        if (self.speed >= 4) {
                            self.speed /= 2;
                        } else if self.speed >= 1 {
                            self.speed -= 1;
                        }
                    } else if btn == "slower" && self.speed <= 10000 {
                        if (self.speed >= 2) {
                            self.speed *= 2;
                        } else {
                            self.speed += 2;
                        }
                    } else if btn == "pause" {
                        self.paused = !self.paused;
                    } else if btn == "step" {
                        self.step = true;
                    }
                },
                ComputerMessage::GetData() => {
                    let l = self.processor.info.len();
                    if l > 20 {
                        self.processor.info = self.processor.info[l-20..].to_vec();
                    }
                    //println!("{:?}", self.processor);
                    self.processor.test = self.data[0x200];
                    self.tx.send(
                        ControllerMessage::UpdatedProcessorAvailable(self.processor.clone())
                    );
                    //only send a slice of the data
                    let btm :u16 = if self.processor.pc > 256 { (self.processor.pc - 255) }else {0};
                    let top :u16 = if (self.processor.pc < 0xffff - 256) { self.processor.pc + 256} else { 0xffff };
                    let mem_to_display = self.data[btm as usize ..=top as usize].to_vec();

                    self.tx.send(
                        ControllerMessage::UpdatedDataAvailable(mem_to_display)
                    );

                    let stack = self.data[0x100 as usize..=0x1ff as usize].to_vec();

                    self.tx.send(
                        ControllerMessage::UpdatedStackAvailable(stack)
                    );
                },
            };
        }

        if self.paused && !self.step {
            thread::sleep(time::Duration::from_millis(1000));
            return true;
        }

        if (self.paused && self.step) || !self.paused {
            self.step = false;
            let changed = self.run_instruction();
            if self.speed > 0 {
                thread::sleep(time::Duration::from_millis(self.speed));
            }
        }

        true
    }

    fn run_instruction(&mut self) {
        let inst = self.data[(self.processor.pc) as usize];

        match inst {
            
            0x10 => {
                //// println!("Running instruction bpl : {:x?}", inst);
                self.bpl();
            },
            0x18 => {
                //// println!("Running instruction clc : {:x?}", inst);
                self.clc();
            },
            0x30 => {
                //// println!("Running instruction clc : {:x?}", inst);
                self.bmi();
            },
            0x48 => {
                //// println!("Running instruction eor : {:x?}", inst);
                self.pha();
            },
            0x49 => {
                //// println!("Running instruction eor : {:x?}", inst);
                self.eor();
            },
            0x4c => {
                //// println!("Running instruction jmp : {:x?}", inst);
                self.jmp();
            },
            0x69 => {
                //// println!("Running instruction adc : {:x?}", inst);
                self.adc();
            },
            0x88 => {
                //// println!("Running instruction dey : {:x?}", inst);
                self.dey();
            },
            0x8a => {
                //// println!("Running instruction dey : {:x?}", inst);
                self.txa();
            },
            0x8d => {
                //// println!("Running instruction sta : {:x?}", inst);
                self.sta();
            },
            0x90 => {
                //// println!("Running instruction bpl : {:x?}", inst);
                self.bcc();
            },
            0x98 => {
                //// println!("Running instruction tya : {:x?}", inst);
                self.tya();
            },
            0x9a => {
                //// println!("Running instruction txs : {:x?}", inst);
                self.txs();
            },
            0xa0 => {
                //// println!("Running instruction ldy : {:x?}", inst);
                self.ldy();
            },
            0xa2 => {
                //// println!("Running instruction ldx : {:x?}", inst);
                self.ldx();
            },
            0xa8 => {
                //// println!("Running instruction ldx : {:x?}", inst);
                self.tay();
            },
            0xaa => {
                //// println!("Running instruction tax : {:x?}", inst);
                self.tax();
            },
            0xa9 | 0xad => {
                //// println!("Running instruction lda : {:x?}", inst);
                self.lda();
            },
            0xb0 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.bcs();
            },
            0xba => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.tsx();
            },
            0xc0 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.cpy();
            },
            0xc9 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.cmp(ADRESSING_MODE::IMMEDIATE);
            },
            0xca => {
                //// println!("Running instruction dex : {:x?}", inst);
                self.dex();
            },
            0xcd => {
                //// println!("Running instruction dex : {:x?}", inst);
                self.cmp(ADRESSING_MODE::ABSOLUTE);
            },
            0xd0 => {
                //// println!("Running instruction bne : {:x?}", inst);
                self.bne();
            },
            0xd8 => {
                //// println!("Running instruction cld : {:x?}", inst);
                self.cld();
            },
            0xe0 => {
                //// println!("Running instruction bne : {:x?}", inst);
                self.cpx();
            },
            0xf0 => {
                //// println!("Running instruction beq : {:x?}", inst);
                self.beq();
            },
            _ => {
                //// println!("Running instruction nop : {:x?}", inst);
                self.nop();
            },
        };
    }

    fn add_info(&mut self, info: String) {
        let len = self.processor.info.len();
        if len > 0 && self.processor.info[len-1].msg == info {
            let last_element = self.processor.info.pop().unwrap();
            self.processor.info.push(Info {msg: info, qty: last_element.qty + 1});
        } else {
            self.processor.info.push(Info {msg: info, qty: 1});
        }
    }

    fn adc(&mut self) {
        let mut acc = self.processor.acc;
        let val = self.data[(self.processor.pc + 1) as usize];
        acc += val;
        self.processor.flags = Self::set_flags(self.processor.flags, acc);
        self.add_info(format!("{:#x} - Running instruction adc: {:#x} with acc: {:#x} memval: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.acc, val));
        self.processor.acc = acc;
        self.processor.clock += 2;
        self.processor.pc += 2;
    }

    fn cld(&mut self) {
        self.add_info(format!("{:#x} - Running instruction cld: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.flags = self.processor.flags & 0xF7;
        self.processor.clock += 2;
    }

    fn txs(&mut self) {
        self.add_info(format!("{:#x} - Running instruction txs: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.sp = self.processor.rx;
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
    }

    fn tsx(&mut self) {
        self.add_info(format!("{:#x} - Running instruction tsx: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.rx = self.processor.sp;
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
    }

    fn tya(&mut self) {
        self.add_info(format!("{:#x} - Running instruction tya: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.ry;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
    }

    fn tay(&mut self) {
        self.add_info(format!("{:#x} - Running instruction tay: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.ry = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
    }

    fn tax(&mut self) {
        self.add_info(format!("{:#x} - Running instruction tax: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.rx = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
    }

    fn txa(&mut self) {
        self.add_info(format!("{:#x} - Running instruction txa: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.rx;
    }

    fn clc(&mut self) {
        self.processor.flags =  self.processor.flags & 0xFE;
        self.add_info(format!("{:#x} - Running instruction clc: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn pha(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        self.processor.sp -= 1;

        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.acc;
        self.data = _addr;

        self.add_info(format!("{:#x} - Running instruction pha at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        self.processor.pc += 1;
        self.processor.clock += 3;
    }

    fn eor(&mut self) {
        let val = self.data[(self.processor.pc + 1) as usize];
        let mut acc = self.processor.acc;
        self.add_info(format!("{:#x} - Running instruction eor: {:#x} with acc: {:#x} memval: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], acc, val));
        //// println!("EOR {:x?} {:x?}", val, acc);
        
        acc ^= val;
        self.processor.pc += 2;
        self.processor.acc = acc;
    }

    fn ldx(&mut self) {
        let x = self.data[(self.processor.pc + 1) as usize];
        self.add_info(format!("{:#x} - Running instruction ldx: {:#x} with val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], x));
        self.processor.rx = x;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.pc += 2;
    }

    fn ldy(&mut self) {
        
        let y = self.data[(self.processor.pc + 1) as usize];
        self.add_info(format!("{:#x} - Running instruction ldy: {:#x} with val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], y));
        self.processor.ry = y;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        self.processor.pc += 2;
        self.processor.clock += 4;
        
    }

    fn lda(&mut self) {
        let mut acc = self.data[(self.processor.pc + 1) as usize];
        let inst = self.data[(self.processor.pc) as usize];
        let mut pc = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction lda: {:#x} val: {:#x}", self.processor.pc, inst, acc);
        self.processor.clock += 2;
        if inst == 0xad {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let addr = self.get_word(start);
            //// println!("inst is absolute addr {:x?}", addr);
            acc = self.data[addr as usize];
            pc = self.processor.pc + 3;
            self.processor.clock += 2;
            info = format!("{:#x} - Running instruction lda absolute: {:#x} with addr: {:#x} and val: {:#x}", self.processor.pc, inst, addr, acc);
        }
        self.add_info(info);
        self.processor.acc = acc;
        self.processor.pc = pc;
        self.processor.flags = Self::set_flags(self.processor.flags, acc);
    }

    fn dex(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.add_info(format!("{:#x} - Running instruction dex: new val: {:#x} flags: {:#b}", self.processor.pc, self.processor.rx, self.processor.flags));
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn dey(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags,  self.processor.ry);
        self.add_info(format!("{:#x} - Running instruction dey: {:#x} new val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.ry));
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn cmp(&mut self, adressing_mode: ADRESSING_MODE) {
        let acc = self.processor.acc;
        let mut value: u8 = 0;
        let mut pc = self.processor.pc + 2;
        if adressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[(self.processor.pc + 1) as usize];
        } else if adressing_mode == ADRESSING_MODE::ABSOLUTE {
            let start = self.processor.pc + 1;
            pc += 1;
            let addr = self.get_word(start);
            value = self.data[addr as usize];
        }
        
        let mut flags = self.processor.flags;
        
        //If equal, all flags are zero
        // if a > cmp carry flag is set
        //if cmp > a neg flag is set
        
        if acc == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if (acc > value) {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        self.add_info(format!("{:#x} - Running instruction cmp: {:#x} with acc: {:#x} val: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], acc, value, flags));

        self.processor.flags = flags;
        self.processor.pc = pc;
        self.processor.clock += 4;
        
    }

    fn cpy(&mut self) {
        let ry = self.processor.ry;
        let value = self.data[(self.processor.pc + 1) as usize];
        
        let mut flags = self.processor.flags;

        if ry == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if (ry > value) {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        self.add_info(format!("{:#x} - Running instruction cpy ry: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, ry, value, flags));

        self.processor.flags = flags;
        self.processor.pc += 2;
        self.processor.clock += 4;
    }

    fn cpx(&mut self) {
        let rx = self.processor.rx;
        let value = self.data[(self.processor.pc + 1) as usize];
        
        let mut flags = self.processor.flags;

        if rx == value {
            flags |= FLAG_Z | FLAG_C;
            flags &= !FLAG_N;
        } else if (rx > value) {
            flags |= FLAG_C;
            flags &= !(FLAG_N | FLAG_Z);
        } else {
            flags |= FLAG_N;
            flags &= !(FLAG_C | FLAG_Z);
        }
        self.add_info(format!("{:#x} - Running instruction cpx rx: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, rx, value, flags));

        self.processor.flags = flags;
        self.processor.pc += 2;
        self.processor.clock += 4;
    }

    fn sta(&mut self) {
        let addr = self.get_word(self.processor.pc + 1);
    // // println!("sta addr 0x{:x?}", addr);
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.acc;
        self.data = _addr;

        self.add_info(format!("{:#x} - Running instruction sta: {:#x} at: {:#x} val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], addr, self.processor.acc));
        self.processor.pc += 3;
        self.processor.clock += 5;
    }

    fn jmp(&mut self) {
        let addr = self.get_word(self.processor.pc + 1);
        self.add_info(format!("{:#x} - Running instruction jmp: {:#x} to: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], addr));
        //// println!("Jumping to 0x{:x?}", addr);
        self.processor.pc = addr;
    }

    fn bne(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];

        let should_jump = (self.processor.flags >> 1) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction bne NOT jumping to: {:#x} flags: {:#b}", self.processor.pc, new_addr, self.processor.flags);

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction bne {:#x} jumping to: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags);
        }

        self.processor.clock += 3;
        self.processor.pc = new_addr;
        self.add_info(info);
    }

    /// Branch if not equal
    fn beq(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_Z != 0;
        let mut new_addr :u16 = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction beq not jumping to: {:#x} flags: {:#b}", self.processor.pc, new_addr, self.processor.flags);

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction beq {:#x} jumping to: {:#x} flags: {:#b} offset {}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags, offset as i8);
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
        self.add_info(info);
    }

    /// Branch if carry clear
    fn bcc(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_C == 0;
        let mut new_addr = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction bcc NOT jumping to: {:#x} flags: {:#b} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8);
        
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction bcc jumping to: {:#x} flags: {:#b} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8);
        }
        self.add_info(info);
        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }

    /// Branch if carry set
    fn bcs(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags) & FLAG_C == 1;
        let mut new_addr :u16 = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction bcs not jumping to: {:#x} flags: {:#b}", self.processor.pc, new_addr, self.processor.flags);

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction bcs {:#x} jumping to: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags);
        }
        self.add_info(info);
        self.processor.clock += 3;
        self.processor.pc = new_addr;
        
    }

    fn bpl(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction bpl not jumping: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.flags);
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction bpl {:#x} jumping to: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags);
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
        self.add_info(info);
    }

    /// Branch if negative flag is set
    fn bmi(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 1;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("{:#x} - Running instruction bmi not jumping: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.flags);
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("{:#x} - Running instruction bmi {:#x} jumping to: {:#x} flags: {:#b}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags);
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
        self.add_info(info);
    }

    fn nop(&mut self) {
        let inst = self.data[(self.processor.pc) as usize];
        if (inst != 0xea) {
            self.speed = 1000;
        }
        self.add_info(format!("{:#x} - Running instruction nop: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        
    }

    pub fn set_flags(flags:u8, val:u8) -> u8 {
        let mut _flags = flags;
        if val == 0 {
            //Set zero flag
            _flags |= FLAG_Z & !FLAG_N;
        } else {
            _flags &= !FLAG_Z;
        }
        if (val >> 7 == 1) {
            _flags |= FLAG_N;
        }else {
            _flags &= !FLAG_N;
        }
        // // println!("Setting flags to {:#b}", _flags);
        return _flags;
    }

    pub fn get_word(&mut self, address: u16) -> u16 {
        let low_byte :u16 = self.data[(address) as usize].into();
        let high_byte :u16 = self.data[(address + 1) as usize].into();
        return low_byte + (high_byte << 8);
    }
}
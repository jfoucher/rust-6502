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
    GetData(),
    UpdatedProcessorAvailable(Processor),
    UpdatedDataAvailable(Vec<u8>),
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
    rx: mpsc::Receiver<ControllerMessage>,
}

impl Computer {
    pub fn new(tx: mpsc::Sender<ControllerMessage>, rx:  mpsc::Receiver<ControllerMessage>, data: Vec<u8>) -> Computer {
        let mut computer = Computer {
            data,
            tx,
            rx,
            paused: true,
            step: false,
            speed: 1000,
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
                ControllerMessage::ButtonPressed(btn) => {
                    if btn == "faster" && self.speed >= 2 {
                        self.speed /= 2;
                    } else if btn == "slower" && self.speed <= 10000 {
                        self.speed *= 2;
                    } else if btn == "pause" {
                        self.paused = !self.paused;
                    } else if btn == "step" {
                        self.step = true;
                    }
                },
                ControllerMessage::UpdatedProcessorAvailable(processor) => {},
                ControllerMessage::UpdatedDataAvailable(data) => {},
                ControllerMessage::GetData() => {
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
                },
            };
        }

        if self.paused && !self.step {
            return true;
        }

        if (self.paused && self.step) || !self.paused {
            self.step = false;
            let changed = self.run_instruction();

            thread::sleep(time::Duration::from_millis(self.speed));
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
            0x8d => {
                //// println!("Running instruction sta : {:x?}", inst);
                self.sta();
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
            0xaa => {
                //// println!("Running instruction tax : {:x?}", inst);
                self.tax();
            },
            0xa9 | 0xad => {
                //// println!("Running instruction lda : {:x?}", inst);
                self.lda();
            },
            0xc0 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.cpy();
            },
            0xc9 => {
                //// println!("Running instruction cmp : {:x?}", inst);
                self.cmp();
            },
            0xca => {
                //// println!("Running instruction dex : {:x?}", inst);
                self.dex();
            },
            0xd0 => {
                //// println!("Running instruction bne : {:x?}", inst);
                self.bne();
            },
            0xd8 => {
                //// println!("Running instruction cld : {:x?}", inst);
                self.cld();
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
        self.add_info(format!("Running instruction adc: {:#x} with acc: {:#x} memval: {:#x}", self.data[(self.processor.pc) as usize], self.processor.acc, val));
        self.processor.acc = acc;
        self.processor.clock += 2;
        self.processor.pc += 2;
    }

    fn cld(&mut self) {
        self.add_info(format!("Running instruction cld: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.flags = self.processor.flags & 0x7;
        self.processor.clock += 2;
    }

    fn txs(&mut self) {
        self.add_info(format!("Running instruction txs: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.sp = self.processor.rx;
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
        
    }

    fn tya(&mut self) {
        self.add_info(format!("Running instruction tya: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.ry;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        
    }

    fn clc(&mut self) {
        self.processor.flags =  self.processor.flags & 0xFE;
        self.add_info(format!("Running instruction clc: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 1;
    }

    fn tax(&mut self) {
        self.add_info(format!("Running instruction tax: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
        self.processor.pc += 1;
        self.processor.clock += 1;
        self.processor.rx = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        
    }

    fn eor(&mut self) {
        let val = self.data[(self.processor.pc + 1) as usize];
        let mut acc = self.processor.acc;
        self.add_info(format!("Running instruction eor: {:#x} with acc: {:#x} memval: {:#x}", self.data[(self.processor.pc) as usize], acc, val));
        //// println!("EOR {:x?} {:x?}", val, acc);
        
        acc ^= val;
        self.processor.pc += 2;
        self.processor.acc = acc;
    }

    fn ldx(&mut self) {
        let x = self.data[(self.processor.pc + 1) as usize];
        self.add_info(format!("Running instruction ldx: {:#x} with val: {:#x}", self.data[(self.processor.pc) as usize], x));
        self.processor.rx = x;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.pc += 2;
        
    }

    fn ldy(&mut self) {
        self.add_info(format!("Running instruction ldy: {:#x}", self.data[(self.processor.pc) as usize]));
        let y = self.data[(self.processor.pc + 1) as usize];
        self.processor.ry = y;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        self.processor.pc += 2;
        self.processor.clock += 4;
        
    }

    fn lda(&mut self) {
        let mut acc = self.data[(self.processor.pc + 1) as usize];
        let inst = self.data[(self.processor.pc) as usize];
        let mut pc = self.processor.pc + 2;
        let mut info = format!("Running instruction lda: {:#x}", inst);
        self.processor.clock += 2;
        if inst == 0xad {
            //Absolute adressing

            let addr = Self::get_word(&self.data, self.processor.pc + 1);
            //// println!("inst is absolute addr {:x?}", addr);
            acc = self.data[addr as usize];
            pc = self.processor.pc + 3;
            self.processor.clock += 2;
            info = format!("Running instruction lda absolute: {:#x}", inst);
        }
        self.add_info(info);
        self.processor.pc = pc;
        self.processor.flags = Self::set_flags(self.processor.flags, acc);
    }

    fn dex(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.add_info(format!("Running instruction dex: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn dey(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags,  self.processor.ry);
        self.add_info(format!("Running instruction dey: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn cmp(&mut self) {
        self.add_info(format!("Running instruction cmp: {:#x}", self.data[(self.processor.pc) as usize]));
        let acc = self.processor.acc;
        let value = self.data[(self.processor.pc + 1) as usize];
        let result: u8 = acc.wrapping_sub(value);
        let mut flags = self.processor.flags;
        if (acc > value) {
            flags |= 1;
        }
        flags = Self::set_flags(flags, result as u8);
        self.processor.flags = flags;
        self.processor.pc += 2;
        self.processor.clock += 4;
        
    }

    fn cpy(&mut self) {
        self.add_info(format!("Running instruction cpy: {:#x}", self.data[(self.processor.pc) as usize]));
        let ry = self.processor.ry;
        let value = self.data[(self.processor.pc + 1) as usize];
        let result: u8 = ry.wrapping_sub(value);
        let mut flags = self.processor.flags;
        if (ry > value) {
            flags |= 1;
        }
        flags = Self::set_flags(flags, result as u8);
        self.processor.flags = flags;
        self.processor.pc += 2;
        self.processor.clock += 4;
        
    }

    fn sta(&mut self) {
        let addr = Self::get_word(&self.data, self.processor.pc + 1);
    // // println!("sta addr 0x{:x?}", addr);
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.acc;
        self.data = _addr;

        self.add_info(format!("Running instruction sta: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 3;
        self.processor.clock += 5;
    }

    fn jmp(&mut self) {
        self.add_info(format!("Running instruction jmp: {:#x}", self.data[(self.processor.pc) as usize]));
        let addr = Self::get_word(&self.data, self.processor.pc + 1);
        //// println!("Jumping to 0x{:x?}", addr);
        self.processor.pc = addr;
    }

    fn bne(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];

        let should_jump = (self.processor.flags >> 1) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("Running instruction bne not jumping: {:#x}", self.data[(self.processor.pc) as usize]);

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction bne {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }

        self.processor.clock += 3;
        self.processor.pc = new_addr;
        self.add_info(info);
    }


    fn beq(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 1) & 1 == 1;
        let mut new_addr :u16;
        let mut info = format!("Running instruction beq not jumping: {:#x}", self.data[(self.processor.pc) as usize]);
        new_addr = self.processor.pc + 2;
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction beq {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
        self.add_info(info);
    }

    fn bpl(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        let mut info = format!("Running instruction bpl not jumping: {:#x}", self.data[(self.processor.pc) as usize]);
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            info = format!("Running instruction bpl {:#x} jumping to: {:#x}", self.data[(self.processor.pc) as usize], new_addr);
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
        self.add_info(info);
    }

    fn nop(&mut self) {
        self.add_info(format!("Running instruction nop: {:#x}", self.data[(self.processor.pc) as usize]));
        self.processor.pc += 1;
        self.processor.clock += 2;
        
    }

    pub fn set_flags(flags:u8, val:u8) -> u8 {
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

    pub fn get_word(data: &Vec<u8>, address: u16) -> u16 {
        let low_byte :u16 = data[(address) as usize].into();
        let high_byte :u16 = data[(address + 1) as usize].into();
        return low_byte + (high_byte << 8);
    }
}
use std::sync::mpsc;
use std::time;
use std::thread;

mod decode;
#[derive(Clone, Debug)]
pub struct Info {
    pub msg: String,
    pub qty: u64,
}

const LOG_LEVEL:i16 = 0;

const OUTPUT_BTM:u16 = 0xf000;
const OUTPUT_TOP:u16 = 0xf100;

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub enum ADRESSING_MODE {
    IMMEDIATE = 0,
    ZERO_PAGE = 1,

    ZERO_PAGE_X = 2,
    ABSOLUTE = 3,
    ABSOLUTE_X = 4,
    ABSOLUTE_Y = 5,
    INDIRECT_X = 6,
    INDIRECT_Y = 7,
    INDIRECT = 8,
    ZERO_PAGE_Y = 9,
    ACCUMULATOR = 10,
    NONE = 11,
}

pub enum ControllerMessage {
    ButtonPressed(String),
    UpdatedProcessorAvailable(Processor),
    UpdatedDataAvailable(u16, u8),
    UpdatedStackAvailable(Vec<u8>),
    UpdatedOutputAvailable(Vec<u8>),
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
    pub test: Vec<u8>,
    pub info: Vec<Info>,
    pub clock: u64,
    pub inst: u8,
}

#[derive(Debug)]
pub struct Computer {
    processor: Processor,
    paused: bool,
    step: bool,
    start: bool,
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



impl Computer {
    pub fn new(tx: mpsc::Sender<ControllerMessage>, rx:  mpsc::Receiver<ComputerMessage>, data: Vec<u8>) -> Computer {
        let mut computer = Computer {
            data,
            tx,
            rx,
            paused: true,
            start: true,
            step: false,
            speed: 0,
            processor: Processor {
                flags: 0b00110000,
                acc: 0,
                rx: 0,
                ry: 0,
                /// Start at 0x400
                pc: 0x400,
                sp: 0,
                test: vec![],
                info: vec![],
                clock: 0,
                inst: 0xea,
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
                    //if (self.paused && self.step) || self.start {
                        self.start = false;
                        let l = self.processor.info.len();
                        if l > 30 {
                            self.processor.info = self.processor.info[l-30..].to_vec();
                        }
                        //println!("{:?}", self.processor);
                        self.processor.test = self.data[0x200 as usize .. 0x220 as usize].to_vec();
                        self.tx.send(
                            ControllerMessage::UpdatedProcessorAvailable(self.processor.clone())
                        );
    
                        let stack = self.data[0x100 as usize..=0x1ff as usize].to_vec();
    
                        self.tx.send(
                            ControllerMessage::UpdatedStackAvailable(stack)
                        );
    
                        let output = self.data[OUTPUT_BTM as usize..OUTPUT_TOP as usize].to_vec();
    
                        self.tx.send(
                            ControllerMessage::UpdatedOutputAvailable(output)
                        );
                    //}
                },
            };
        }

        if self.paused && !self.step {
            thread::sleep(time::Duration::from_millis(100));
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
        let inst = &self.data[(self.processor.pc) as usize];
        self.processor.inst = *inst;
        let opcode = decode::get_opcode_name(self.processor.inst);

        //self.add_info(format!("{:#x} - running instruction {} ({:#x})", self.processor.pc, opcode, inst));

        match opcode {
            "ADC" => self.adc(),
            "AND" => self.and(),
            "ASL" => self.asl(),
            "BCC" => self.bcc(),
            "BCS" => self.bcs(),
            "BEQ" => self.beq(),
            "BIT" => self.bit(),
            "BMI" => self.bmi(),
            "BNE" => self.bne(),
            "BPL" => self.bpl(),
            "BRK" => self.brk(),
            "BVC" => self.bvc(),
            "BVS" => self.bvs(),
            "CLC" => self.clc(),
            "CLD" => self.cld(),
            "CLI" => self.cli(),
            "CLV" => self.clv(),
            "CMP" => self.cmp(),
            "CPX" => self.cpx(),
            "CPY" => self.cpy(),
            "DEC" => self.dec(),
            "DEX" => self.dex(),
            "DEY" => self.dey(),
            "EOR" => self.eor(),
            "INC" => self.inc(),
            "INX" => self.inx(),
            "INY" => self.iny(),
            "JMP" => self.jmp(),
            "JSR" => self.jsr(),
            "LDA" => self.lda(),
            "LDX" => self.ldx(),
            "LDY" => self.ldy(),
            "LSR" => self.lsr(),
            "NOP" => self.nop(),
            "ORA" => self.ora(),
            "PHA" => self.pha(),
            "PHP" => self.php(),
            "PLA" => self.pla(),
            "PLP" => self.plp(),
            "ROL" => self.rol(),
            "ROR" => self.ror(),
            "RTI" => self.rti(),
            "RTS" => self.rts(),
            "SBC" => self.sbc(),
            "SEC" => self.sec(),
            "SED" => self.sed(),
            "SEI" => self.sei(),
            "STA" => self.sta(),
            "STX" => self.stx(),
            "STY" => self.sty(),
            "TAX" => self.tax(),
            "TAY" => self.tay(),
            "TSX" => self.tsx(),
            "TXA" => self.txa(),
            "TXS" => self.txs(),
            "TYA" => self.tya(),
            
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
            self.paused = true;
        } else {
            self.processor.info.push(Info {msg: info, qty: 1});
        }

    }

    fn cld(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction cld: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.flags = self.processor.flags & !FLAG_D;
        self.processor.clock += 2;
    }

    fn txs(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction txs: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.sp = self.processor.rx;
    }

    fn tsx(&mut self) {
        self.processor.flags = Self::set_flags( self.processor.flags, self.processor.sp);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction tsx: {:#x} val: {:#x} flags:{:#x} ", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.sp, self.processor.flags));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.rx = self.processor.sp;
    }

    fn tya(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction tya: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.ry;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.acc);
    }

    fn tay(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction tay: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.ry = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
    }

    fn tax(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction tax: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.rx = self.processor.acc;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
    }

    fn txa(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction txa: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        self.processor.pc += 1;
        self.processor.clock += 2;
        self.processor.acc = self.processor.rx;
    }

    /// Jump to subroutine
    fn jsr(&mut self) {
        // Place current address on stack
        let sp: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        let sp1: u16 = (self.processor.sp.wrapping_sub(1) as u16 + 0x100 as u16).into();
        let mut _addr = self.data.to_vec();
        let this_pc = self.processor.pc + 2;
        _addr[sp as usize] = ((this_pc>>8) & 0xff) as u8;
        _addr[sp1 as usize] = (this_pc & 0xff) as u8;
        self.data = _addr;
        // Send to new address
        let addr = self.get_word(self.processor.pc + 1);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction jsr to: {:#x}", self.processor.pc, addr));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(2);
        self.processor.pc = addr;
    }

    fn brk(&mut self) {
        let sp: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        let sp1: u16 = (self.processor.sp.wrapping_sub(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_sub(2) as u16 + 0x100 as u16).into();
        let mut _addr = self.data.to_vec();
        let this_pc = self.processor.pc + 2;
        _addr[sp as usize] = ((this_pc>>8) & 0xff) as u8;
        _addr[sp1 as usize] = (this_pc & 0xff) as u8;
        
        _addr[sp2 as usize] = (self.processor.flags) | 0x30;

        self.processor.flags |= FLAG_I;
        self.processor.sp = self.processor.sp.wrapping_sub(3);
        
        self.data = _addr;

        let new_addr: u16 = self.get_word(0xfffe);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction brk ({:#x}) to: {:#x} flags: {:#b}", self.processor.pc, self.processor.inst, new_addr, self.processor.flags));
        }
        self.processor.pc = new_addr;

        self.processor.clock += 7;
    }

    fn rti(&mut self) {
        // Place current address on stack
        let sp1: u16 = (self.processor.sp.wrapping_add(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_add(2) as u16 + 0x100 as u16).into();
        let sp3: u16 = (self.processor.sp.wrapping_add(3) as u16 + 0x100 as u16).into();
        let high_byte = self.data[sp3 as usize];
        let low_byte = self.data[sp2 as usize];
        let flags = self.data[sp1 as usize];
        // Unset interrupt disabled flag
        self.processor.flags = flags;
        let addr: u16 = low_byte as u16 | ((high_byte as u16) << 8) as u16;
        // Send to new address
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction rti to: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags));
        }
        self.processor.sp = self.processor.sp.wrapping_add(3);
        self.processor.pc = addr;
        self.processor.clock += 6;
    }

    fn rts(&mut self) {
        // Place current address on stack
        let sp1: u16 = (self.processor.sp.wrapping_add(1) as u16 + 0x100 as u16).into();
        let sp2: u16 = (self.processor.sp.wrapping_add(2) as u16 + 0x100 as u16).into();
        let low_byte = self.data[sp1 as usize];
        let high_byte = self.data[sp2 as usize];
        let addr: u16 = low_byte as u16 | ((high_byte as u16) << 8) as u16;
        // Send to new address
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction rts to: {:#x}", self.processor.pc, addr));
        }
        self.processor.sp = self.processor.sp.wrapping_add(2);
        self.processor.pc = addr + 1;
        self.processor.clock += 6;
    }

    /// Clear carry flag
    fn clc(&mut self) {
        self.processor.flags &= !FLAG_C;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction clc: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// Set carry flag
    fn sec(&mut self) {
        self.processor.flags |= FLAG_C;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction sec: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// Set decimal flag
    fn sed(&mut self) {
        self.processor.flags |= FLAG_D;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction sed: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// Clear interrupt disabled flag
    fn cli(&mut self) {
        self.processor.flags &= !FLAG_I;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction cli: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// Set interrupt disabled flag
    fn sei(&mut self) {
        self.processor.flags |= FLAG_I;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction sei: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// clear overflow flag
    fn clv(&mut self) {
        self.processor.flags &= !FLAG_O;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction clv: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    /// Push accumulator to stack
    fn pha(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.acc;
        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr,  self.processor.acc)
        );
        self.data = _addr;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction pha at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc += 1;
        self.processor.clock += 3;
    }

    /// Push flags to stack
    fn php(&mut self) {
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.flags | 0x30;
        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr,  self.processor.flags | 0x30)
        );
        self.data = _addr;
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction php at: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags | 0x30));
        }
        self.processor.sp = self.processor.sp.wrapping_sub(1);
        self.processor.pc += 1;
        self.processor.clock += 3;
    }

    /// Pull stack to accumulator
    fn pla(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.acc = self.data[addr as usize];
        let flags = self.processor.flags;
        self.processor.flags = Self::set_flags(flags, self.processor.acc);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction pla at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
        }
        self.processor.pc += 1;
        self.processor.clock += 4;
    }

    // 0X28 Pull value from the stack into the processor registers
    fn plp(&mut self) {
        self.processor.sp = self.processor.sp.wrapping_add(1);
        let addr: u16 = (self.processor.sp as u16 + 0x100 as u16).into();
        
        self.processor.flags = self.data[addr as usize];
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction plp at: {:#x} flags: {:#x}", self.processor.pc, addr, self.processor.flags));
        }
        self.processor.pc += 1;
        self.processor.clock += 4;
    }


    fn get_ld_adddr(&mut self, addressing_mode: ADRESSING_MODE) -> u16 {
        if LOG_LEVEL > 3 {
            let inst = self.data[self.processor.pc as usize];
            self.add_info(format!("{:#x} - Getting address with mode {:?} for inst {:#x}", self.processor.pc, addressing_mode, inst));
        }

        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            return self.processor.pc + 1;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let addr = self.get_word(start);
            return addr;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE_X {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.get_word(start);
            let rx = self.processor.rx;
            let addr: u16 = start_addr + (rx as u16);
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting absolute_x address from: {:#x} ry: {:#x} gives: {:#x}", self.processor.pc, start_addr, rx, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.get_word(start);
            let ry = self.processor.ry;
            let addr: u16 = start_addr + (ry as u16);
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting absolute_y address from: {:#x} ry: {:#x} gives: {:#x}", self.processor.pc, start_addr, ry, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let addr: u16 = self.data[start as usize].into();
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE address from: {:#x} gives: {:#x}", self.processor.pc, start, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE_Y {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.data[start as usize].wrapping_add(self.processor.ry);
            let addr: u16 = start_addr.into();
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE_Y address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let start_addr = self.data[start as usize].wrapping_add(self.processor.rx);
            let addr: u16 = start_addr.into();
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting ZERO_PAGE_X address from: {:#x} with rx: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.rx, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::INDIRECT_Y {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let zp_addr = self.data[start as usize];
            let base_addr = self.get_word(zp_addr.into());
            let addr: u16 = base_addr + self.processor.ry as u16;
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting INDIRECT_Y address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        } else if addressing_mode == ADRESSING_MODE::INDIRECT_X {
            //Absolute adressing
            let start = self.processor.pc + 1;
            let zp_addr = self.data[start as usize].wrapping_add(self.processor.rx);
            let addr: u16 = self.get_word(zp_addr.into());
            
            if LOG_LEVEL > 2 {
                self.add_info(format!("{:#x} - Getting INDIRECT_X address from: {:#x} with ry: {:#x} gives: {:#x}", self.processor.pc, start, self.processor.ry, addr));
            }
            return addr;
        }

        return 0;
    }

    fn inc(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);
        if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction inc ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 5;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction inc ABS with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 3;
            self.processor.clock += 6;
        }

        let result = value.wrapping_add(1);

        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = result;
        self.data = _addr;
        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr, result)
        );

        self.processor.flags = Self::set_flags(self.processor.flags, result);
    }

    fn dec(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);
        if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction dec ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 5;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction dec ABS with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 3;
            self.processor.clock += 6;
        }

        let result = value.wrapping_sub(1);

        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr, result)
        );

        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = result;
        self.data = _addr;

        self.processor.flags = Self::set_flags(self.processor.flags, result);
    }

    fn ldx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);

        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx val: {:#x}", self.processor.pc, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 2;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X || addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx absolute with addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 3;
            self.processor.clock += 4;
        }else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_Y {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldx ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 3;
        }
        self.processor.rx = value;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
    }

    fn ldy(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;
        let addr = self.get_ld_adddr(mode);

        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy val: {:#x}", self.processor.pc, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 2;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X || addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy absolute with addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 3;
            self.processor.clock += 4;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction ldy ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 3;
        }

        self.processor.ry = value;
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        
    }

    fn lda(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u8 = 0;
        let mode = addressing_mode;
        let addr = self.get_ld_adddr(mode);
        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction lda val: {:#x}", self.processor.pc, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 2;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X|| addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction lda absolute with addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 3;
            self.processor.clock += 4;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction lda ZP with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 3;
        } else if addressing_mode == ADRESSING_MODE::INDIRECT_Y || addressing_mode == ADRESSING_MODE::INDIRECT_X {
            value = self.data[addr as usize];
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction lda INDIRECT with effective addr: {:#x} and val: {:#x}", self.processor.pc, addr, value));
            }
            self.processor.pc += 2;
            self.processor.clock += 5;
        } else {
            panic!("This adressing mode is not implemented yet, sorry");
        }
        
        self.processor.acc = value;
        self.processor.flags = Self::set_flags(self.processor.flags, value);
    }

    fn asl(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let mut value: u8 = 0;
        let addr = self.get_ld_adddr(mode);
        if (mode == ADRESSING_MODE::ACCUMULATOR) {
            value = self.processor.acc;
            self.processor.pc += 1;
            self.processor.clock += 2;
        } else if mode == ADRESSING_MODE::ABSOLUTE || mode == ADRESSING_MODE::ABSOLUTE_X {
            self.processor.pc += 3;
            self.processor.clock += 6;
            value = self.data[addr as usize];
        } else {
            self.processor.pc += 2;
            self.processor.clock += 6;
            value = self.data[addr as usize];
        }
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if value == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }

        let result = value << 1;
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if (mode == ADRESSING_MODE::ACCUMULATOR) {
            self.processor.acc = result;
        } else {
            self.tx.send(
                ControllerMessage::UpdatedDataAvailable(addr, result)
            );
            let mut _addr = self.data.to_vec();
            _addr[addr as usize] = result;
            self.data = _addr;
        }
    }

    fn lsr(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let mut value: u8 = 0;
        let addr = self.get_ld_adddr(mode);
        if mode == ADRESSING_MODE::ACCUMULATOR {
            value = self.processor.acc;
        } else {
            value = self.data[addr as usize];
        }
        let old_flags = self.processor.flags;
        if value & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if value == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }

        let result = value >> 1;
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction lsr val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if mode == ADRESSING_MODE::ACCUMULATOR {
            self.processor.pc += 1;
            self.processor.clock += 2;
            self.processor.acc = result;
        } else if mode == ADRESSING_MODE::ABSOLUTE || mode == ADRESSING_MODE::ABSOLUTE_X {
            self.processor.pc += 3;
            self.processor.clock += 6;
            self.tx.send(
                ControllerMessage::UpdatedDataAvailable(addr, result)
            );
            let mut _addr = self.data.to_vec();
            _addr[addr as usize] = result;
            self.data = _addr;
        } else {
            self.processor.pc += 2;
            self.processor.clock += 5;
            let mut _addr = self.data.to_vec();
            self.tx.send(
                ControllerMessage::UpdatedDataAvailable(addr, result)
            );
            _addr[addr as usize] = result;
            self.data = _addr;
        }

    }

    fn rol(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let mut value: u8 = 0;
        let addr = self.get_ld_adddr(mode);
        if mode == ADRESSING_MODE::ACCUMULATOR {
            value = self.processor.acc;
            self.processor.pc += 1;
            self.processor.clock += 2;
        } else if mode == ADRESSING_MODE::ABSOLUTE || mode == ADRESSING_MODE::ABSOLUTE_X {
            value = self.processor.acc;
            self.processor.pc += 3;
            self.processor.clock += 6;
        } else {
            self.processor.pc += 2;
            self.processor.clock += 6;
            value = self.data[addr as usize];
        }
        
        let old_flags = self.processor.flags;
        let result = (value << 1) | (self.processor.flags & FLAG_C);
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction rol val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if (mode == ADRESSING_MODE::ACCUMULATOR) {
            self.processor.acc = result;
        } else {
            let mut _addr = self.data.to_vec();
            self.tx.send(
                ControllerMessage::UpdatedDataAvailable(addr, result)
            );
            _addr[addr as usize] = result;
            self.data = _addr;
        }
    }

    fn ror(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let mut value: u8 = 0;
        let addr = self.get_ld_adddr(mode);
        if mode == ADRESSING_MODE::ACCUMULATOR {
            value = self.processor.acc;
            self.processor.pc += 1;
            self.processor.clock += 2;
        } else if mode == ADRESSING_MODE::ABSOLUTE || mode == ADRESSING_MODE::ABSOLUTE_X {
            value = self.processor.acc;
            self.processor.pc += 3;
            self.processor.clock += 6;
        } else {
            self.processor.pc += 2;
            self.processor.clock += 6;
            value = self.data[addr as usize];
        }
        
        let old_flags = self.processor.flags;
        let result = (value >> 1) | ((self.processor.flags & FLAG_C) << 7);
        if value & 1 == 1 {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }
        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if result >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction ror val: {:#x} result: {:#x} flags: {:#x} old flags: {:#x}", self.processor.pc, value, result, self.processor.flags, old_flags));
        }
        if mode == ADRESSING_MODE::ACCUMULATOR {
            self.processor.acc = result;
        } else {
            let mut _addr = self.data.to_vec();
            self.tx.send(
                ControllerMessage::UpdatedDataAvailable(addr, result)
            );
            _addr[addr as usize] = result;
            self.data = _addr;
        }
    }

    fn bit(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mode = addressing_mode;

        let addr = self.get_ld_adddr(mode);
        let value = self.data[addr as usize];

        let result = self.processor.acc & value;

        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction bit val: {:#x} result: {:#x}", self.processor.pc, value, result));
}
        if addressing_mode == ADRESSING_MODE::ZERO_PAGE {
            self.processor.pc += 2;
            self.processor.clock += 2;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            self.processor.pc += 3;
            self.processor.clock += 4;
        } else {
            panic!("Sorry, this adressing mode does not exist for this instruction")
        }

        if result == 0 {
            self.processor.flags |= FLAG_Z;
        } else {
            self.processor.flags &= !FLAG_Z;
        }
        if value >> 7 & 1 == 1 {
            self.processor.flags |= FLAG_N;
        } else {
            self.processor.flags &= !FLAG_N;
        }
        if value >> 6 & 1 == 1 {
            self.processor.flags |= FLAG_O;
        } else {
            self.processor.flags &= !FLAG_O;
        }
    }

    fn inx(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_add(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction inx: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.rx, self.processor.flags));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn iny(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_add(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.ry);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction iny: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.ry, self.processor.flags));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn dex(&mut self) {
        self.processor.rx = self.processor.rx.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags, self.processor.rx);
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction dex: new val: {:#x} flags: {:#x}", self.processor.pc, self.processor.rx, self.processor.flags));
        }
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn dey(&mut self) {
        self.processor.ry = self.processor.ry.wrapping_sub(1);
        self.processor.flags = Self::set_flags(self.processor.flags,  self.processor.ry);
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction dey: {:#x} new val: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], self.processor.ry));
}
        self.processor.pc += 1;
        self.processor.clock += 2;
    }

    fn cmp(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let acc = self.processor.acc;
        let mut value: u8 = 0;
        let mut pc = self.processor.pc + 2;
        let addr = self.get_ld_adddr(addressing_mode);
        value = self.data[addr as usize];
        if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_Y || addressing_mode == ADRESSING_MODE::ABSOLUTE_X {
            pc += 1;
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
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction cmp: {:#x} with acc: {:#x} val: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], acc, value, flags));
}

        self.processor.flags = flags;
        self.processor.pc = pc;
        self.processor.clock += 4;
        
    }

    fn cpy(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let ry = self.processor.ry;
        let mut value: u8 = 0;
        let mut pc = self.processor.pc + 2;
        let addr = self.get_ld_adddr(addressing_mode);
        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[addr as usize];
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            pc += 1;
            value = self.data[addr as usize];
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE {
            value = self.data[addr as usize];
        } else {
            panic!("Unknown address type {:?} {:#b}, {:#x}", addressing_mode, self.processor.inst, self.processor.inst);
        }
        
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
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction cpy ry: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, ry, value, flags));
}

        self.processor.flags = flags;
        self.processor.pc = pc;
        self.processor.clock += 4;
    }

    fn cpx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let rx = self.processor.rx;
        let mut value: u8 = 0;
        let mut pc = self.processor.pc + 2;
        let addr = self.get_ld_adddr(addressing_mode);
        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            value = self.data[addr as usize];
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            pc += 1;
            value = self.data[addr as usize];
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE {
            value = self.data[addr as usize];
        } else {
            panic!("Unknown address type");
        }
        
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
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction cpx rx: {:#x} with val: {:#x} flags: {:#x}", self.processor.pc, rx, value, flags));
}

        self.processor.flags = flags;
        self.processor.pc = pc;
        self.processor.clock += 4;
    }

    fn sta(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut addr: u16 = 0;
        let mut pc = self.processor.pc;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X || addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction sta ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
}

            pc += 3;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X || addressing_mode == ADRESSING_MODE::ZERO_PAGE_Y {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction sta ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
}

            pc += 2;
        } else if addressing_mode == ADRESSING_MODE::INDIRECT_Y || addressing_mode == ADRESSING_MODE::INDIRECT_X {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction sta INDIRECT at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.acc));
}

            pc += 2;
        } else {
            panic!("This adressing mode is not implemented yet, sorry");
        }

        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr, self.processor.acc)
        );
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.acc;
        self.data = _addr;

        self.processor.pc = pc;
        self.processor.clock += 5;
    }

    fn stx(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut addr: u16 = 0;
        let mut pc = 2;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction stx ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
}
            pc = 3;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_Y {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction stx ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
}
        }
        if addr == 0x200 {
            //self.paused = true;
        }
        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr, self.processor.rx)
        );
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.rx;
        self.data = _addr;

        self.processor.pc += pc;
        self.processor.clock += 5;
    }

    fn sty(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut addr: u16 = 0;
        let mut pc = 2;
        let addr = self.get_ld_adddr(addressing_mode);
    // // println!("sta addr 0x{:x?}", addr);
        if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction sty ABS at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
}
            pc = 3;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction sty ZP at: {:#x} val: {:#x}", self.processor.pc, addr, self.processor.rx));
}
        }
        if addr == 0x200 {
            //self.paused = true;
        }
        self.tx.send(
            ControllerMessage::UpdatedDataAvailable(addr, self.processor.ry)
        );
        let mut _addr = self.data.to_vec();
        _addr[addr as usize] = self.processor.ry;
        self.data = _addr;

        self.processor.pc += pc;
        self.processor.clock += 5;
    }

    fn jmp(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let mut value: u16 = 0;
        let mut pc = self.processor.pc + 2;
        if addressing_mode == ADRESSING_MODE::ABSOLUTE {
            value = self.get_word(self.processor.pc + 1);
        } else if addressing_mode == ADRESSING_MODE::INDIRECT {
            let start = self.processor.pc + 1;
            pc += 1;
            let addr = self.get_word(start);
            value = self.get_word(addr);
        } else {
            panic!("Adressing mode not implmented yet");
        }

        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction jmp: {:#x} to: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], value));
}
        //// println!("Jumping to 0x{:x?}", addr);
        self.processor.pc = value;
    }

    fn bne(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];

        let should_jump = (self.processor.flags >> 1) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bne {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bne NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
            }
        }

        self.processor.clock += 3;
        self.processor.pc = new_addr;

        

    }

    /// Branch if not equal
    fn beq(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_Z != 0;
        let mut new_addr :u16 = self.processor.pc + 2;
        

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction beq {:#x} jumping to: {:#x} flags: {:#x} offset {}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags, offset as i8));
            }
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction beq not jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
            }
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
        
    }

    /// Branch if carry clear
    fn bcc(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_C == 0;
        let mut new_addr = self.processor.pc + 2;
        
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bcc jumping to: {:#x} flags: {:#x} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8));
            }
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bcc NOT jumping to: {:#x} flags: {:#x} offset: {}", self.processor.pc, new_addr, self.processor.flags, offset as i8));
            }
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }

    /// Branch if carry set
    fn bcs(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags) & FLAG_C == 1;
        let mut new_addr :u16 = self.processor.pc + 2;

        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bcs {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            } else {
                if LOG_LEVEL > 0 {
                    self.add_info(format!("{:#x} - Running instruction bcs not jumping to: {:#x} flags: {:#x}", self.processor.pc, new_addr, self.processor.flags));
                }
            }
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
        
    }

    /// Branch if overflow clear
    fn bvc(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_O == 0;
        let mut new_addr = self.processor.pc + 2;
        
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bvc {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bvc {:#x} NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        }
        
        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }

    /// Branch if overflow set
    fn bvs(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = self.processor.flags & FLAG_O != 0;
        let mut new_addr = self.processor.pc + 2;
           
        if (should_jump) {
            let rel_address = offset as i8;
            // // println!("Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bvs {:#x} jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }  
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - Running instruction bvs {:#x} NOT jumping to: {:#x} flags: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize], new_addr, self.processor.flags));
            }
        }
        self.processor.clock += 3;
        self.processor.pc = new_addr;
    }

    fn bpl(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 0;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
        
    }

    /// Branch if negative flag is set
    fn bmi(&mut self) {
        let offset = self.data[(self.processor.pc + 1) as usize];
        // println!("Jumping RAW offset is {:?} or 0x{:x?}", offset, offset);
        let should_jump = (self.processor.flags >> 7) & 1 == 1;
        let mut new_addr :u16;
        new_addr = self.processor.pc + 2;
        if (should_jump) {
            let rel_address = offset as i8;
            // println!("BPL Jumping offset {:?}", rel_address);
            new_addr = ((new_addr as i32) + (rel_address as i32)) as u16;
        }
        self.processor.pc = new_addr;
        self.processor.clock += 3;
        
    }

    fn get_logical_op_value(&mut self) -> u8 {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        return self.data[addr as usize];
    }

    fn after_logical_op(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        if addressing_mode == ADRESSING_MODE::IMMEDIATE {
            self.processor.pc += 2;
            self.processor.clock += 2;
        } else if addressing_mode == ADRESSING_MODE::ZERO_PAGE || addressing_mode == ADRESSING_MODE::ZERO_PAGE_X {
            self.processor.pc += 2;
            self.processor.clock += 3;
        } else if addressing_mode == ADRESSING_MODE::INDIRECT_X || addressing_mode == ADRESSING_MODE::INDIRECT_Y {
            self.processor.pc += 2;
            self.processor.clock += 6;
        } else if addressing_mode == ADRESSING_MODE::ABSOLUTE || addressing_mode == ADRESSING_MODE::ABSOLUTE_X || addressing_mode == ADRESSING_MODE::ABSOLUTE_Y {
            self.processor.pc += 3;
            self.processor.clock += 4;
        } else {
            if LOG_LEVEL > 0 {
                self.add_info(format!("{:#x} - this addressing mode not implemented for instruction {:?}", self.processor.pc, addressing_mode));
            }
        }
    }

    fn and(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc & value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction and with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.acc, value, result, self.processor.flags));
}

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn eor(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc ^ value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction eor {:#x} with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.inst, value, result, self.processor.acc, self.processor.flags));
}

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn ora(&mut self) {
        let value = self.get_logical_op_value();

        let result = self.processor.acc | value;
        self.processor.flags = Self::set_flags(self.processor.flags, result);
        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction ora {:#x} with acc: {:#x} value: {:#x} result: {:#x} flags: {:#x}", self.processor.pc, self.processor.inst, value, result, self.processor.acc, self.processor.flags));
}

        self.processor.acc = result;
        self.after_logical_op();
    }

    fn adc(&mut self) {
        
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        let val = self.data[addr as usize];
        let acc = self.processor.acc;
        let mut carry = self.processor.flags & FLAG_C != 0;
        let decimal = self.processor.flags & FLAG_D != 0;

        let mut sum = 0;

        if (decimal) {
            let mut s: u16 = 0;
            let mut ln = (acc & 0xF) + (val &0xF) + (self.processor.flags & FLAG_C);
            if (ln > 9) {
                ln = 0x10 | ((ln + 6) & 0xf);
            }
            let mut hn: u16 = (acc & 0xf0) as u16 + (val & 0xf0) as u16;
            s = hn + ln as u16;

            

            if (s >= 160) {
                self.processor.flags |= FLAG_C;
                if ((self.processor.flags & FLAG_O) != 0 && s >= 0x180) { self.processor.flags &= !FLAG_O; }
                s += 0x60;
            } else {
                self.processor.flags &= !FLAG_C;
                if ((self.processor.flags & FLAG_O) != 0 && s < 0x80) { self.processor.flags &= !FLAG_O; }
            }
            sum  = (s & 0xff) as u8;
            self.processor.flags = Self::set_flags(self.processor.flags, sum);
        } else {
            sum = self.do_add(val);
        }
        

        if LOG_LEVEL > 0 {
self.add_info(format!("{:#x} - Running instruction adc with acc: {:#x} memval: {:#x} flags: {:#x} carry: {} result: {:#x}", self.processor.pc, self.processor.acc, val, self.processor.flags, carry, sum));
}
        self.processor.acc = sum;
        self.after_logical_op();
    }

    fn sbc(&mut self) {
        let addressing_mode = decode::get_adressing_mode(self.processor.inst);
        let addr = self.get_ld_adddr(addressing_mode);
        let val = self.data[addr as usize];
        let decimal = self.processor.flags & FLAG_D != 0;
        let acc= self.processor.acc;

        let mut carry: bool = false;
        let mut sum = 0;

        if (decimal) {

            // tmp = 0xf + (regA & 0xf) - (value & 0xf) + carrySet();
            // if (tmp < 0x10) {
            // w = 0;
            // tmp -= 6;
            // } else {
            // w = 0x10;
            // tmp -= 0x10;
            // }
            // w += 0xf0 + (regA & 0xf0) - (value & 0xf0);
            // if (w < 0x100) {
            // CLC();
            // if (overflowSet() && w < 0x80) { CLV(); }
            // w -= 0x60;
            // } else {
            // SEC();
            // if (overflowSet() && w >= 0x180) { CLV(); }
            // }
            // w += tmp;


            let mut w: u16;
            let mut tmp = 0xf + (acc & 0xf) - (val & 0xf) + (self.processor.flags & FLAG_C);
            if (tmp < 0x10) {
                w = 0;
              tmp -= 6;
            } else {
              w = 0x10;
              tmp -= 0x10;
            }
            w += 0xf0 + ((acc as u16) & 0xf0) - ((val as u16) & 0xf0);
            if (w < 0x100) {
              self.processor.flags &= !FLAG_C;
              if ((self.processor.flags & FLAG_O) != 0 && w < 0x80) { self.processor.flags &= !FLAG_O; }
              w -= 0x60;
            } else {
                self.processor.flags |= FLAG_C;
              if ((self.processor.flags & FLAG_O) != 0  && w >= 0x180) { self.processor.flags &= !FLAG_O; }
            }
            w += tmp as u16;
            sum = w as u8
        } else {
            sum = self.do_add(!val);
        }

        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction sbc with acc: {:#x} memval: {:#x} flags: {:#x}", self.processor.pc, self.processor.acc, val, self.processor.flags));
        }
        self.processor.acc = sum as u8;
        self.after_logical_op();
    }

    fn do_add(&mut self, val: u8) -> u8 {
        let mut acc = self.processor.acc;
        let mut carry = false;
        let bit7 = acc >> 7;
        let s = acc as u16 + val as u16 + (self.processor.flags & FLAG_C) as u16;
        
        if s > 255 {
            carry = true;
        }
        let mut sum = 0;
    
        sum = acc.wrapping_add(val);
        //Add carry bit
        sum = sum.wrapping_add(self.processor.flags & FLAG_C);
        self.processor.flags = Self::set_flags(self.processor.flags, sum);

        if carry {
            self.processor.flags |= FLAG_C;
        } else {
            self.processor.flags &= !FLAG_C;
        }

        if (acc ^ sum) & (val ^ sum) & 0x80 != 0 {
            self.processor.flags |= FLAG_O;
        } else {
            self.processor.flags &= !FLAG_O;
        }


        

        return sum;
    }

    fn nop(&mut self) {
        if LOG_LEVEL > 0 {
            self.add_info(format!("{:#x} - Running instruction nop: {:#x}", self.processor.pc, self.data[(self.processor.pc) as usize]));
        }
        if (self.processor.inst != 0xea) {
            self.speed = 1000;
        }
        
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

        _flags |= 0x30;

        // // println!("Setting flags to {:#b}", _flags);
        return _flags;
    }

    pub fn get_word(&mut self, address: u16) -> u16 {
        let low_byte :u16 = self.data[(address) as usize].into();
        let high_byte :u16 = self.data[(address + 1) as usize].into();
        return low_byte + (high_byte << 8);
    }
}
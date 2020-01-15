extern crate cursive;
use std::env;
use std::io;
use std::io::Write;
use std::fs;
use cursive::Cursive;
use cursive::theme::Effect;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

mod computer;
mod utils;

use computer::{Processor, Computer, ControllerMessage, ComputerMessage};

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    ui_tx: mpsc::Sender<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
    data: Vec<u8>,
    clk: u64,
    t: u128,
}

pub enum UiMessage {
    UpdateProcessor(Processor),
    UpdateData(u16, u8),
    FullData(Vec<u8>),
    UpdateStack(Vec<u8>),
    UpdateOutput(Vec<u8>),
}

impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> Ui {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let t = SystemTime::now().duration_since(UNIX_EPOCH).expect("fail");
        let mut ui = Ui {
            cursive: Cursive::default(),
            ui_tx: ui_tx,
            ui_rx: ui_rx,
            controller_tx: controller_tx,
            data: vec![],
            clk: 0,
            t: t.as_millis(),
        };

        // Create a view tree with a TextArea for input, and a
        // TextView for output.
        let controller_tx_clone = ui.controller_tx.clone();
        let controller_tx_clone1 = ui.controller_tx.clone();
        let controller_tx_clone2 = ui.controller_tx.clone();
        let controller_tx_clone3 = ui.controller_tx.clone();

        ui.cursive.add_layer(
            Dialog::around(
                utils::layout()
            )
            
            .button("Faster", move |s| {
                controller_tx_clone.send(
                    ControllerMessage::ButtonPressed("faster".to_string())
                )
                .unwrap();
            })
            .button("Slower", move |s| {
                controller_tx_clone1.send(
                    ControllerMessage::ButtonPressed("slower".to_string())
                )
                .unwrap();
            })
            .button("Pause", move |s| {
                controller_tx_clone2.send(
                    ControllerMessage::ButtonPressed("pause".to_string())
                )
                .unwrap();
            })
            .button("Step", move |s| {
                controller_tx_clone3.send(
                    ControllerMessage::ButtonPressed("step".to_string())
                )
                .unwrap();
            })
            .button("Quit", |s| {
                std::process::abort();
                std::process::exit(0);
            })
            .title("6502 simulator")
            .full_screen()
        );

        // Configure a callback
        ui.cursive.refresh();
        
        ui
    }

    /// Step the UI by calling into Cursive's step function, then
    /// processing any UI messages.
    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        let mut prev_info: String;
        // Process any pending UI messages
        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::UpdateProcessor(processor) => {
                    //println!("UpdateProcessor {}", processor.clock);
                    let mut output = self.cursive
                        .find_id::<TextView>("flags")
                        .unwrap();
                    output.set_content(format!("{:08b}", processor.flags));

                    let mut output = self.cursive
                        .find_id::<TextView>("pc")
                        .unwrap();
                    output.set_content(format!("{} ({:#06x})", processor.pc, processor.pc));
                    let mut output = self.cursive
                        .find_id::<TextView>("acc")
                        .unwrap();
                    output.set_content(format!("{:#04x}", processor.acc));
                    let mut output = self.cursive
                        .find_id::<TextView>("rx")
                        .unwrap();
                    output.set_content(format!("{:#04x}", processor.rx));
                    let mut output = self.cursive
                        .find_id::<TextView>("ry")
                        .unwrap();
                    output.set_content(format!("{:#04x}", processor.ry));
                    let mut output = self.cursive
                        .find_id::<TextView>("sp")
                        .unwrap();
                    output.set_content(format!("{:#04x}", processor.sp));

                    let mut output = self.cursive
                        .find_id::<TextView>("clock")
                        .unwrap();
                    output.set_content(format!("{}", processor.clock));
                    
                    let start = SystemTime::now().duration_since(UNIX_EPOCH).expect("fail");
                    let t = start.as_millis();


                    if t - self.t > 1000 {
                        let sp = processor.clock - self.clk;
                        self.clk = processor.clock;
                        self.t = t;
                        let mut speed = self.cursive
                            .find_id::<TextView>("speed")
                            .unwrap();
                        speed.set_content(format!("{:.2} MHz", sp as f64 / 1000000.0));
                    }
                    

                    let mut info = self.cursive
                        .find_id::<TextView>("info")
                        .unwrap();
                    
                    
                    let mut v = processor.info;
                    
                    v.reverse();
                    
                    let r: Vec<String> = v.iter().map(|l| {
                        let qty = l.qty.clone();
                        if qty <= 1 {
                            return l.msg.clone();
                        }
                        format!("{} ({})", l.msg.clone(), l.qty.clone())
                    }).collect();
                    
                    info.set_content(r.join("\n"));

                    let mut output = self.cursive
                        .find_id::<TextView>("test")
                        .unwrap();
                    output.set_content(format!("{}", processor.test[0]));

                    // chunk data
                    
                    //let out = (&self.data).chunks(16);
                    let out: Vec<&[u8]> = self.data.chunks(16).collect();
                    let center = processor.pc >> 4;

                    //println!("center: {:#x} - number of lines : {}", center, out.len());

                    let btm: u16 = if center - 16 < 0 { 0 } else { center - 16 };
                    let top: u16 = if center + 16 > out.len() as u16 { out.len() as u16 } else { center + 16 };

                    let mut iter = out[btm as usize ..=top as usize].iter();
                    let mut cnt = 0;

                    while let Some(line) = iter.next() {
                        let mut inner = line.iter();
                        let mut text = "".to_string();

                        let atv = self.cursive.find_id::<TextView>(format!("addr-{}", cnt >> 4).as_str());
                        let addr = (btm << 4) + cnt;
                        while let Some(item) = inner.next() {
                            
                            if cnt % 4 == 0 {
                                text = format!("{}  {:02x}", text.as_str(), item);
                            } else {
                                text = format!("{} {:02x}", text.as_str(), item);
                            }
                            cnt += 1;
                        }

                        if let Some(mut aview) = atv {
                            aview.set_content(format!("{:#06x}  {}", addr, text));
                        }

                    }
                    // Update memory display here

                    // break full data by lines of 32 bytes
                    // Center vertically on processor.pc
                },
                UiMessage::UpdateData(addr, data) => {
                    self.data[addr as usize] = data;
                },
                UiMessage::UpdateStack(data) => {
                    let mut cnt = 0;
                    let out: Vec<&[u8]> = data.chunks(16).collect();
                    let mut iter = out.iter();

                    while let Some(line) = iter.next() {
                        let mut inner = line.iter();
                        let mut text = "".to_string();

                        let atv = self.cursive.find_id::<TextView>(format!("stack-{}", cnt >> 4).as_str());
                        let addr = 0x100 + cnt;

                        while let Some(item) = inner.next() {
                            
                            if cnt % 4 == 0 {
                                text = format!("{}  {:02x}", text.as_str(), item);
                            } else {
                                text = format!("{} {:02x}", text.as_str(), item);
                            }
                            cnt += 1;
                        }

                        if let Some(mut aview) = atv {
                            aview.set_content(format!("{:#06x}  {}", addr, text));
                        }
                    }
                },
                UiMessage::UpdateOutput(data) => {
                    if let Some(mut output) = self.cursive.find_id::<TextView>("output") {
                        let mut cnt = 0;
                        let out: Vec<&[u8]> = data.chunks(16).collect();
                        let mut iter = out.iter();
                        let mut text = "".to_string();
                        while let Some(line) = iter.next() {
                            let mut inner = line.iter();
                            
                            let addr = 0xf000 + cnt;
                            text = format!("{}{:#06x}  ", text.as_str(), addr);
                            while let Some(item) = inner.next() {
                                if cnt % 4 == 0 {
                                    text = format!("{}  {:02x}", text.as_str(), item);
                                } else {
                                    text = format!("{} {:02x}", text.as_str(), item);
                                }
                                cnt += 1;
                            }
                            text = format!("{}\n", text.as_str());
                        }
                        output.set_content(text);
                    }   
                },
                UiMessage::FullData(data) => {
                    self.data = data;
                },
                _ => {},
            }
        }

        // Step the UI
        self.cursive.step();
        self.cursive.refresh();
        true
    }
}


pub struct Controller {
    rx: mpsc::Receiver<ControllerMessage>,
    ctx: mpsc::Sender<ComputerMessage>,
    ui: Ui,
}

impl Controller {
    /// Create a new controller
    pub fn new(filename: String) -> Result<Controller, String> {
        let data = fs::read(filename).expect("could not read file");
        
        let (tx, rx) = mpsc::channel::<ControllerMessage>();
        let controller_tx = tx.clone();
        let (computer_tx, computer_rx) = mpsc::channel::<ComputerMessage>();
        let computer_data = data.clone();
        let child = thread::spawn(move || {
            let mut computer = Computer::new(controller_tx, computer_rx, computer_data);
            loop {
                computer.step();
            }
        });
        

        let ui = Ui::new(tx.clone());

        ui
            .ui_tx
            .send(UiMessage::FullData(data))
            .unwrap();

        Ok(Controller {
            rx: rx,
            ctx: computer_tx.clone(),
            ui,
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        let mut speed = 2;
        let mut i = 1;
        let mut paused: bool = true;
        let mut step: bool = false;
        while self.ui.step() {
            self.ctx.send(ComputerMessage::GetData());
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::ButtonPressed(btn) => {
                        self.ctx.send(ComputerMessage::ButtonPressed(btn));
                    },
                    ControllerMessage::UpdatedProcessorAvailable(processor) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateProcessor(processor))
                            .unwrap();
                        //self.computer.step();
                    },
                    ControllerMessage::UpdatedDataAvailable(addr, data) => {
                        // self.ui
                        //     .ui_tx
                        //     .send(UiMessage::UpdateData(data))
                        //     .unwrap();
                    },

                    ControllerMessage::UpdatedStackAvailable(data) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateStack(data))
                            .unwrap();
                    },
                    ControllerMessage::UpdatedOutputAvailable(data) => {
                            self.ui
                            .ui_tx
                            .send(UiMessage::UpdateOutput(data))
                            .unwrap();
                    },
                };
            }
        }
    }
}

fn main() {
    // Launch the controller and UI
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Please enter a filename to run");
    }
    let filename = &args[1];

    let controller = Controller::new(filename.to_string());
    match controller {
        Ok(mut controller) => controller.run(),
        Err(e) => println!("Error: {}", e),
    };
}
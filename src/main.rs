extern crate cursive;
use std::env;

use std::fs;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;
use std::thread;

mod computer;
mod utils;

use computer::{Processor, Computer, ControllerMessage};

pub struct Ui {
    cursive: Cursive,
    ui_rx: mpsc::Receiver<UiMessage>,
    ui_tx: mpsc::Sender<UiMessage>,
    controller_tx: mpsc::Sender<ControllerMessage>,
}

pub enum UiMessage {
    UpdateProcessor(Processor),
    UpdateData(Vec<u8>),
}

impl Ui {
    /// Create a new Ui object.  The provided `mpsc` sender will be used
    /// by the UI to send messages to the controller.
    pub fn new(controller_tx: mpsc::Sender<ControllerMessage>) -> Ui {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let mut ui = Ui {
            cursive: Cursive::default(),
            ui_tx: ui_tx,
            ui_rx: ui_rx,
            controller_tx: controller_tx,
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
                    output.set_content(format!("{:b}", processor.flags));

                    let mut output = self.cursive
                        .find_id::<TextView>("pc")
                        .unwrap();
                    output.set_content(format!("{} ({:#x})", processor.pc, processor.pc));
                    let mut output = self.cursive
                        .find_id::<TextView>("acc")
                        .unwrap();
                    output.set_content(format!("{}", processor.acc));
                    let mut output = self.cursive
                        .find_id::<TextView>("rx")
                        .unwrap();
                    output.set_content(format!("{}", processor.rx));
                    let mut output = self.cursive
                        .find_id::<TextView>("ry")
                        .unwrap();
                    output.set_content(format!("{}", processor.ry));
                    let mut output = self.cursive
                        .find_id::<TextView>("sp")
                        .unwrap();
                    output.set_content(format!("{}", processor.sp));
                    let mut output = self.cursive
                        .find_id::<TextView>("clock")
                        .unwrap();
                    output.set_content(format!("{}", processor.clock));

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
                    output.set_content(format!("{}", processor.test));
                },
                UiMessage::UpdateData(data) => {
                    let mut output = self.cursive
                        .find_id::<TextView>("memory")
                        .unwrap();
                    output.set_content(format!("{:x?}", data));
                    
                },
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
    ctx: mpsc::Sender<ControllerMessage>,
    ui: Ui,
}

impl Controller {
    /// Create a new controller
    pub fn new(filename: String) -> Result<Controller, String> {
        let data = fs::read(filename).expect("could not read file");
        let (tx, rx) = mpsc::channel::<ControllerMessage>();
        let controller_tx = tx.clone();
        let (computer_tx, computer_rx) = mpsc::channel::<ControllerMessage>();
        let child = thread::spawn(move || {
            let mut computer = Computer::new(controller_tx, computer_rx, data);
            loop {
                computer.step();
            }
        });
        

        Ok(Controller {
            rx: rx,
            ctx: computer_tx.clone(),
            ui: Ui::new(tx.clone()),
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        let mut speed = 2;
        let mut i = 1;
        let mut paused: bool = true;
        let mut step: bool = false;
        while self.ui.step() {
            self.ctx.send(ControllerMessage::GetData());
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::ButtonPressed(btn) => {
                        self.ctx.send(ControllerMessage::ButtonPressed(btn));
                    },
                    ControllerMessage::UpdatedProcessorAvailable(processor) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateProcessor(processor))
                            .unwrap();
                        //self.computer.step();
                    },
                    ControllerMessage::UpdatedDataAvailable(data) => {
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateData(data))
                            .unwrap();
                    },

                    ControllerMessage::GetData() => {},
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
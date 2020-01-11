extern crate cursive;
use std::env;

use std::fs;
use cursive::Cursive;
use cursive::event::Key;
use cursive::view::*;
use cursive::views::*;
use std::sync::mpsc;

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
        ui.cursive.add_layer(
            Dialog::around(
                LinearLayout::horizontal()
                .child(Dialog::around(
                    TextView::new("TEST MEM").with_id("memory")
                ))
                .child(Dialog::around(
                    TextView::new("PROC DATA").with_id("processor")
                ))
            )
            .button("Quit", |s| {
                std::process::abort();
                std::process::exit(0);
            })
            .button("Faster", move |s| {
                controller_tx_clone.send(
                    ControllerMessage::ButtonPressed("faster".to_string())
                )
                .unwrap();
            })
            .title("6502 simulator")
            .full_screen()
        );

        // Configure a callback
        
        
        ui
    }

    /// Step the UI by calling into Cursive's step function, then
    /// processing any UI messages.
    pub fn step(&mut self) -> bool {
        if !self.cursive.is_running() {
            return false;
        }

        // Process any pending UI messages
        while let Some(message) = self.ui_rx.try_iter().next() {
            match message {
                UiMessage::UpdateProcessor(processor) => {
                    let mut output = self.cursive
                        .find_id::<TextView>("processor")
                        .unwrap();
                    output.set_content(format!("{:?}", processor));
                },
                UiMessage::UpdateData(data) => {
                    let mut output = self.cursive
                        .find_id::<TextView>("memory")
                        .unwrap();
                    output.set_content(format!("{:?}", data));
                },
            }
        }

        // Step the UI
        self.cursive.step();
        self.cursive.refresh();
        true
    }
}

#[derive(Clone, Debug)]
pub struct Processor {
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

#[derive(Clone, Debug)]
pub struct Computer {
    data: Vec<u8>,
    processor: Processor,
    tx: mpsc::Sender<ControllerMessage>
}

impl Computer {
    pub fn new(tx: mpsc::Sender<ControllerMessage>, data: Vec<u8>) -> Computer {
        let mut computer = Computer {
            data,
            tx,
            processor: Processor {
                flags: 0,
                acc: 0,
                rx: 0,
                ry: 0,
                pc: 0,
                sp: 0,
                info: "".to_string(),
                clock: 0,
                speed: 0,
            }
        };
        computer
    }

    pub fn step(&mut self) -> bool {
        // Process any pending UI messages
        self.processor.clock += 1;
        self.tx.send(
            ControllerMessage::UpdatedProcessorAvailable(self.processor.to_owned())
        );

        true
    }
}

pub struct Controller {
    rx: mpsc::Receiver<ControllerMessage>,
    ui: Ui,
    computer: Computer,
}

pub enum ControllerMessage {
    ButtonPressed(String),
    UpdatedProcessorAvailable(Processor),
    UpdatedDataAvailable(Vec<u8>),
}

impl Controller {
    /// Create a new controller
    pub fn new(filename: String) -> Result<Controller, String> {
        let data = fs::read(filename).expect("could not read file");
        let (tx, rx) = mpsc::channel::<ControllerMessage>();
        Ok(Controller {
            rx: rx,
            ui: Ui::new(tx.clone()),
            computer: Computer::new(tx.clone(), data),
        })
    }
    /// Run the controller
    pub fn run(&mut self) {
        while self.ui.step() {
            self.computer.step();
            while let Some(message) = self.rx.try_iter().next() {
                // Handle messages arriving from the UI.
                match message {
                    ControllerMessage::ButtonPressed(btn) => {
                        println!("button pressed {}", btn)
                    },
                    ControllerMessage::UpdatedProcessorAvailable(processor) => {
                        //println!("processor updated");
                        self.ui
                            .ui_tx
                            .send(UiMessage::UpdateProcessor(processor))
                            .unwrap();
                    },
                    ControllerMessage::UpdatedDataAvailable(data) => {
                        println!("data");
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
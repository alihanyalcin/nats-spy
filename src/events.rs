use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::thread;

pub enum InputEvent<I> {
    Input(I),
    Tick,
}

pub struct Events {
    keyboard_rx: Receiver<InputEvent<Event>>,
}

impl Events {
    pub fn new() -> Events {
        let (keyboard_tx, keyboard_rx) = channel();

        thread::spawn(move || loop {
            if let Ok(key) = read() {
                if let Err(err) = keyboard_tx.send(InputEvent::Input(key)) {
                    eprintln!("{}", err);
                    return;
                }
            }
        });

        Events { keyboard_rx }
    }

    pub fn next(&self) -> Result<InputEvent<Event>, RecvError> {
        self.keyboard_rx.recv()
    }
}

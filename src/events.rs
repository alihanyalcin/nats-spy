use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError};

pub enum InputEvent<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent<Event>>,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            // let stdout = stdout();
            loop {
                if let Ok(key) = read() {
                    if let Err(err) = tx.send(InputEvent::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                }
            }
        });

        Events { rx }
    }

    pub fn next(&self) -> Result<InputEvent<Event>, RecvError> {
        self.rx.recv()
    }
}

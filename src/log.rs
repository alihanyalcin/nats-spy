use crate::events::InputEvent;
use std::sync::mpsc::Sender;

pub struct Log {
    tx: Sender<InputEvent>,
}

impl Log {
    pub fn new(tx: Sender<InputEvent>) -> Self {
        Self { tx }
    }

    pub fn info(&self, msg: String) {
        self.log(format!("INFO: {}", msg))
    }

    pub fn error(&self, msg: String) {
        self.log(format!("ERROR: {}", msg))
    }

    fn log(&self, msg: String) {
        if let Err(err) = self.tx.send(InputEvent::Logs(msg)) {
            eprintln!("{}", err);
        }
    }
}

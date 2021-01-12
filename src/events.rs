use crate::log::Log;
use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::thread;

pub enum InputEvent {
    Input(Event),
    Logs(String),
}

pub struct Events {
    rx: Receiver<InputEvent>,
    log: Log,
}

impl Events {
    pub fn new() -> Events {
        let (tx, rx) = channel();

        let tx_keyboard = tx.clone();
        thread::spawn(move || loop {
            if let Ok(key) = read() {
                if let Err(err) = tx_keyboard.send(InputEvent::Input(key)) {
                    eprintln!("{}", err);
                    return;
                }
            }
        });

        let log = Log::new(tx);

        Events { rx, log }
    }

    pub fn next_key(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }

    pub fn connect(
        &mut self,
        host: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) {
        self.log.info("Trying to connect...".to_string());

        let mut nats_client = NatsClient::new(host.clone(), username, password, token);

        match nats_client.connect() {
            Ok(_) => self.log.info(format!("Connected to {}", host)),
            Err(err) => self.log.error(format!("Not connected. {}", err)),
        }
    }
}

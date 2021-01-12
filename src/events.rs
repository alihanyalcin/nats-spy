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
    nc: NatsClient,
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
        let nc = NatsClient::default();

        Events { rx, log, nc }
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
        self.log
            .info(format!("Trying to connect NATS Server {}", host));

        let mut nats_client = NatsClient::new(host.clone(), username, password, token);

        match nats_client.connect() {
            Ok(_) => {
                self.nc = nats_client;
                self.log.info(format!("Connected to {}", host))
            }
            Err(err) => self.log.error(format!("Not connected. {}", err)),
        }
    }

    pub fn drain(&mut self) {
        self.nc.drain()
    }
}

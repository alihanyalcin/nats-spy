use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::thread;

#[derive(Clone)]
pub enum InputEvent {
    Input(Event),
    Logs(String),
}

pub struct Events {
    rx: Receiver<InputEvent>,
    nats_client: NatsClient,
}

impl Events {
    pub fn new(nats_url: String) -> Events {
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

        let nats_client = NatsClient::new(nats_url, None, None, None);

        let mut nc = nats_client.clone();
        let tx_log = tx.clone();
        thread::spawn(move || {
            tx_log
                .send(InputEvent::Logs(
                    "Tring to connect NATS Server...".to_string(),
                ))
                .unwrap();

            match nc.connect() {
                Ok(_) => tx_log
                    .send(InputEvent::Logs("Connected to NATS Server".to_string()))
                    .unwrap(),
                Err(err) => tx_log
                    .send(InputEvent::Logs(format!("Not connected. {}", err)))
                    .unwrap(),
            }
        });

        Events { rx, nats_client }
    }

    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }

    pub fn drain(&mut self) {
        self.nats_client.drain()
    }
}

use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;

pub enum InputEvent<I> {
    Input(I),
    Logs(String),
    Tick,
}

pub struct Events<'a> {
    keyboard_rx: Receiver<InputEvent<Event>>,
    logs: &'a mut Vec<String>,
}

impl<'a> Events<'a> {
    pub fn new(logs: &mut Vec<String>) -> Events {
        let (keyboard_tx, keyboard_rx) = channel();

        thread::spawn(move || loop {
            if let Ok(key) = read() {
                if let Err(err) = keyboard_tx.send(InputEvent::Input(key)) {
                    eprintln!("{}", err);
                    return;
                }
            }
        });

        Events { keyboard_rx, logs }
    }

    pub fn next_key(&self) -> Result<InputEvent<Event>, RecvError> {
        self.keyboard_rx.recv()
    }

    pub fn connect(
        &mut self,
        host: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) {
        let mut nats_client = NatsClient::new(host.clone(), username, password, token);

        match nats_client.connect() {
            Ok(_) => self
                .logs
                .push(format!("Connected to NATS server {}", host).to_string()),
            Err(err) => self
                .logs
                .push(format!("Client cannot connected. Error: {}", err).to_string()),
        }
    }
}

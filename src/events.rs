use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use log::{error, info};
use std::sync::mpsc::{channel, Receiver, RecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub enum InputEvent {
    Input(Event),
    Messages(String),
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    nats_client: Arc<Mutex<NatsClient>>,
}

impl Events {
    pub fn new(
        nats_url: String,
        subject: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) -> Events {
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

        let tx_tick = tx.clone();
        thread::spawn(move || loop {
            if let Err(err) = tx_tick.send(InputEvent::Tick) {
                eprintln!("{}", err);
                return;
            }
            thread::sleep(Duration::from_millis(250));
        });

        let nats_client = Arc::new(Mutex::new(NatsClient::new(
            nats_url, username, password, token,
        )));
        let nc = nats_client.clone();
        let tx_message = tx.clone();
        thread::spawn(move || {
            info!("Trying to connect NATS Server...");

            let mut nc = nc.lock().unwrap();
            match nc.connect() {
                Ok(_) => info!("Connected to NATS Server"),
                Err(err) => {
                    error!("Not connected. {}", err);
                    return;
                }
            }

            let sub = match nc.subscribe(subject) {
                Ok(sub) => sub,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };
            drop(nc);

            for msg in sub.messages() {
                tx_message
                    .send(InputEvent::Messages(format!(
                        "[{}] -> {}",
                        msg.subject,
                        std::str::from_utf8(&msg.data).unwrap()
                    )))
                    .unwrap()
            }
        });

        Events { rx, nats_client }
    }

    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }

    pub fn publish(&self, sub: String, msg: String) {
        match self.nats_client.lock().unwrap().publish(sub.clone(), msg) {
            Ok(_) => info!("Message send to subject '{}'", sub.clone()),
            Err(err) => error!("Message cannot send to subject '{}'. {}", sub, err),
        }
    }

    pub fn drain(&mut self) {
        self.nats_client.lock().unwrap().drain()
    }
}

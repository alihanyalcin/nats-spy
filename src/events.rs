use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub enum InputEvent {
    Input(Event),
    Logs(String),
    Messages(String),
}

pub struct Events {
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    nats_client: Arc<Mutex<NatsClient>>,
}

impl Events {
    pub fn new(nats_url: String, subject: String) -> Events {
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

        let nats_client = Arc::new(Mutex::new(NatsClient::new(nats_url, None, None, None)));

        let nc = nats_client.clone();
        let tx_log = tx.clone();
        thread::spawn(move || {
            tx_log
                .send(InputEvent::Logs(
                    "Trying to connect NATS Server...".to_string(),
                ))
                .unwrap();

            let mut nc = nc.lock().unwrap();
            match nc.connect() {
                Ok(_) => tx_log
                    .send(InputEvent::Logs("Connected to NATS Server".to_string()))
                    .unwrap(),
                Err(err) => {
                    tx_log
                        .send(InputEvent::Logs(format!("Not connected. {}", err)))
                        .unwrap();

                    return;
                }
            }

            let sub = match nc.subscribe(subject) {
                Ok(sub) => sub,
                Err(err) => {
                    tx_log.send(InputEvent::Logs(err.to_string())).unwrap();
                    return;
                }
            };
            drop(nc);

            for msg in sub.messages() {
                tx_log
                    .send(InputEvent::Messages(format!(
                        "[{}] -> {}",
                        msg.subject,
                        std::str::from_utf8(&msg.data).unwrap()
                    )))
                    .unwrap()
            }
        });

        Events {
            rx,
            tx,
            nats_client,
        }
    }

    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }

    pub fn publish(&self, sub: String, msg: String) {
        match self.nats_client.lock().unwrap().publish(sub.clone(), msg) {
            Ok(_) => self
                .tx
                .send(InputEvent::Logs(format!(
                    "Message send to subject '{}'",
                    sub.clone()
                )))
                .unwrap(),
            Err(err) => self
                .tx
                .send(InputEvent::Logs(format!(
                    "Message cannot send to subject '{}'. {}",
                    sub, err
                )))
                .unwrap(),
        }
    }

    pub fn drain(&mut self) {
        self.nats_client.lock().unwrap().drain()
    }
}

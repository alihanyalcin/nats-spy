use crate::nats::NatsClient;
use anyhow::Result;
use crossterm::event::{read, Event};
use log::{error, info};
use std::{
    sync::{
        mpsc::{channel, Receiver, RecvError, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

#[derive(Clone)]
pub enum InputEvent {
    Input(Event),
    Messages(String, String),
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    nats_client: Arc<Mutex<NatsClient>>,
}

impl Events {
    pub fn new(
        nats_url: String,
        subject: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
        credentials: Option<String>,
    ) -> Events {
        let (tx, rx) = channel();

        // listen keyboard events
        let tx_keyboard = tx.clone();
        thread::spawn(move || loop {
            if let Ok(key) = read() {
                if let Err(err) = tx_keyboard.send(InputEvent::Input(key)) {
                    eprintln!("{}", err);
                    return;
                }
            }
        });

        // start ticker event
        let tx_tick = tx.clone();
        thread::spawn(move || loop {
            if let Err(err) = tx_tick.send(InputEvent::Tick) {
                eprintln!("{}", err);
                return;
            }
            thread::sleep(Duration::from_millis(200));
        });

        let nats_client = Arc::new(Mutex::new(NatsClient::new(
            nats_url,
            username,
            password,
            token,
            credentials,
        )));

        // start nats client and listen
        let nc = nats_client.clone();
        let tx_message = tx.clone();
        thread::spawn(move || {
            info!("Trying to connect NATS Server...");

            // connect nats server
            let mut nc = nc.lock().unwrap();
            match nc.connect() {
                Ok(_) => info!("Connected to NATS Server."),
                Err(err) => {
                    error!("Cannot connect. {}", err);
                    return;
                }
            }

            // subscribe subject
            let sub = match nc.subscribe(subject) {
                Ok(sub) => sub,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };
            drop(nc);

            // listen new messages
            for msg in sub.messages() {
                tx_message
                    .send(InputEvent::Messages(
                        msg.subject,
                        std::str::from_utf8(&msg.data).unwrap().to_string(),
                    ))
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
        if sub.is_empty() {
            error!("Subject is empty!");
            return;
        }

        match self.nats_client.lock().unwrap().publish(sub.clone(), msg) {
            Ok(_) => info!("Message send to subject '{}'", sub.clone()),
            Err(err) => error!("{}", err),
        }
    }

    pub fn request(&self, sub: String, msg: String) {
        if sub.is_empty() {
            error!("Subject is empty!");
            return;
        }

        info!("Subject '{}' requested.", sub.clone());
        match self.nats_client.lock().unwrap().request(sub, msg) {
            Ok(resp) => self
                .tx
                .send(InputEvent::Messages(
                    resp.subject,
                    std::str::from_utf8(&resp.data).unwrap().to_string(),
                ))
                .unwrap(),
            Err(err) => {
                error!("{}", err)
            }
        }
    }

    pub fn drain(&mut self) {
        self.nats_client.lock().unwrap().drain()
    }
}

use anyhow::{bail, Result};
use log::{info, warn};
use nats::{self, Connection, Subscription};

#[derive(Clone)]
pub struct NatsClient {
    host: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    client: Option<Connection>,
    status: ConnectionStatus,
}

#[derive(Clone)]
enum ConnectionStatus {
    Connected,
    Disconnected,
}

impl NatsClient {
    pub fn new(
        host: String,
        token: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        Self {
            host,
            username,
            password,
            token,
            client: None,
            status: ConnectionStatus::Disconnected,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if let ConnectionStatus::Connected = self.status {
            bail!("Already connected.")
        }

        let client = {
            match ((&self.username, &self.password), &self.token) {
                ((Some(username), Some(password)), _) => {
                    nats::Options::with_user_pass(username.as_str(), password.as_str())
                }
                (_, Some(token)) => nats::Options::with_token(token.as_str()),
                _ => nats::Options::new(),
            }
        }
        .with_name("nats-spy")
        .disconnect_callback(|| warn!("disconnect"))
        .reconnect_callback(|| info!("reconnect"))
        .max_reconnects(10)
        .connect(self.host.as_str())?;

        self.client = Some(client);
        self.status = ConnectionStatus::Connected;

        Ok(())
    }

    pub fn drain(&mut self) {
        if let Some(c) = &self.client {
            c.drain().unwrap()
        }
    }

    pub fn subscribe(&self, subject: String) -> Result<Subscription> {
        if let Some(c) = &self.client {
            match c.subscribe(subject.as_str()) {
                Ok(sub) => return Ok(sub),
                Err(err) => bail!("Cannot subscribe. {}", err),
            }
        }
        bail!("no connection")
    }

    pub fn publish(&self, subject: String, message: String) -> Result<()> {
        if subject.is_empty() {
            bail!("Subject is empty")
        }

        if let Some(c) = &self.client {
            c.publish(subject.as_str(), message)?
        } else {
            bail!("no connection")
        }

        Ok(())
    }
}

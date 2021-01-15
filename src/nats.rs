use anyhow::{bail, Result};
use log::{info, warn};
use nats::{self, Connection, Subscription};

#[derive(Clone)]
pub struct NatsClient {
    url: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    credentials: Option<String>,
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
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
        credentials: Option<String>,
    ) -> Self {
        Self {
            url: host,
            username,
            password,
            token,
            credentials,
            client: None,
            status: ConnectionStatus::Disconnected,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        if let ConnectionStatus::Connected = self.status {
            bail!("Already connected.")
        }

        let client = {
            match (
                (&self.username, &self.password),
                &self.token,
                &self.credentials,
            ) {
                ((Some(username), Some(password)), _, _) => {
                    nats::Options::with_user_pass(username.as_str(), password.as_str())
                }
                (_, Some(token), _) => nats::Options::with_token(token.as_str()),
                (_, _, Some(credentials)) => nats::Options::with_credentials(credentials.as_str()),
                _ => nats::Options::new(),
            }
        }
        .with_name("nats-spy")
        .disconnect_callback(|| warn!("Connecction has been lost"))
        .reconnect_callback(|| info!("Connection has been reestablished"))
        .max_reconnects(10)
        .connect(self.url.as_str())?;

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

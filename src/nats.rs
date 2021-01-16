use anyhow::{bail, Result};
use log::{info, warn};
use nats::{self, Connection, Message, Subscription};

#[derive(Clone)]
pub struct NatsClient {
    url: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    credentials: Option<String>,
    client: Option<Connection>,
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
        }
    }

    pub fn connect(&mut self) -> Result<()> {
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
        bail!("Connection cannot established")
    }

    pub fn publish(&self, subject: String, message: String) -> Result<()> {
        if let Some(c) = &self.client {
            if subject.is_empty() {
                bail!("Subject is empty")
            }
            c.publish(subject.as_str(), message)?
        } else {
            bail!("Connection cannot established")
        }

        Ok(())
    }

    pub fn request(&self, subject: String, message: String) -> Result<Message> {
        if let Some(c) = &self.client {
            if subject.is_empty() {
                bail!("Subject is empty")
            }
            info!("Subject '{}' requested.", subject.clone());
            match c.request_timeout(subject.as_str(), message, std::time::Duration::from_secs(2)) {
                Ok(resp) => return Ok(resp),
                Err(err) => bail!("Request {}", err),
            }
        }
        bail!("Connection cannot established")
    }
}

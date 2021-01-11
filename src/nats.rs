use anyhow::Result;
use nats::{self, Connection};

pub struct NatsClient {
    host: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    client: Option<Connection>,
    status: ConnectionStatus,
}

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
        .connect(self.host.as_str())?;

        self.client = Some(client);

        Ok(())
    }

    pub fn publish(&self, topic: String, message: String) {
        match &self.client {
            Some(c) => match c.publish(topic.as_str(), message) {
                Ok(_) => println!("send"),
                Err(_) => println!("not send"),
            },
            None => println!("no connection"),
        }
    }
}
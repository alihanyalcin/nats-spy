mod application;
mod events;
mod keys;
mod nats;

use crate::application::Application;
use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use scopeguard::defer;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> Result<()> {
    let config = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("nats-url")
                .help("nats url")
                .short("n")
                .long("nats-url")
                .default_value("nats://localhost:4222"),
        )
        .arg(
            Arg::with_name("subject")
                .help("subscription subject")
                .short("s")
                .long("subject")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let nats_url = config.value_of("nats-url").unwrap();
    let subject = config.value_of("subject").unwrap();

    // initialize terminal
    setup_terminal()?;
    defer! {
        stop_terminal().expect("stop_terminal error");
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // start terminal
    let mut app = Application::new(nats_url, subject);
    app.draw(&mut terminal)?;

    Ok(())
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    Ok(())
}

fn stop_terminal() -> Result<()> {
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

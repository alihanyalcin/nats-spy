mod application;
mod events;
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
use tui_logger::{init_logger, set_default_level};

fn main() -> Result<()> {
    init_logger(log::LevelFilter::Info).unwrap();
    set_default_level(log::LevelFilter::Info);

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
                .default_value(">"),
        )
        .arg(
            Arg::with_name("username")
                .help("nats username")
                .short("u")
                .long("username"),
        )
        .arg(
            Arg::with_name("password")
                .help("nats password")
                .short("p")
                .long("password"),
        )
        .arg(
            Arg::with_name("token")
                .help("nats token")
                .short("t")
                .long("token"),
        )
        .arg(
            Arg::with_name("credentials")
                .help("nats credentials")
                .short("c")
                .long("credentials"),
        )
        .get_matches();

    let nats_url = config.value_of("nats-url").unwrap();
    let subject = config.value_of("subject").unwrap();
    let username = config.value_of("username");
    let password = config.value_of("password");
    let token = config.value_of("token");
    let credentials = config.value_of("credentials");

    // initialize terminal
    setup_terminal()?;
    defer! {
        stop_terminal().expect("stop_terminal error");
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // start terminal
    let mut app = Application::new(
        nats_url.to_string(),
        subject.to_string(),
        username.map(str::to_string),
        password.map(str::to_string),
        token.map(str::to_string),
        credentials.map(str::to_string),
    );
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

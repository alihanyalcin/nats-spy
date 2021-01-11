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
    init_cmdline()?;

    // initialize terminal
    setup_terminal()?;
    defer! {
        stop_terminal().expect("stop_terminal error");
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // start terminal
    let mut app = Application::new();
    app.draw(&mut terminal)?;

    Ok(())
}

fn init_cmdline() -> Result<()> {
    let app = App::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(Arg::with_name("test").help("test").short("t").long("test"));

    let config = app.get_matches();

    if config.is_present("test") {
        println!("test");
    }

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

use crate::events::{Events, InputEvent};
use crate::keys::KeyConfig;
use anyhow::Result;
use crossterm::event::Event;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Terminal,
};

#[derive(Default)]
pub struct Application {}

impl Application {
    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.clear()?;

        let events = Events::new();
        let keys = KeyConfig::default();

        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(10),
                            Constraint::Percentage(80),
                            Constraint::Percentage(10),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let block = Block::default().title("Block").borders(Borders::ALL);
                f.render_widget(block, chunks[0]);
                let block = Block::default().title("Block 2").borders(Borders::ALL);
                f.render_widget(block, chunks[2]);
            })?;

            if let InputEvent::Input(input) = events.next()? {
                if let Event::Key(e) = input {
                    if e == keys.exit_key {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

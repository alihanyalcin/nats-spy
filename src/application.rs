use anyhow::Result;
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
        }
    }
}

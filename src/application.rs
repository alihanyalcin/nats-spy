use crate::events::{Events, InputEvent};
use crate::keys::KeyConfig;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct Application {
    nats_server: String,
    input_mode: InputMode,
}

impl Application {
    pub fn new() -> Self {
        Self {
            nats_server: String::new(),
            input_mode: InputMode::Normal,
        }
    }

    pub fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.clear()?;

        let events = Events::new();
        let keys = KeyConfig::default();

        loop {
            terminal.draw(|f| {
                // main chunk
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.size());

                // left chunk
                let left_chunk = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Percentage(50)].as_ref())
                    .split(chunks[0]);

                let input_nats_server = Paragraph::new(self.nats_server.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("NATS Server"));
                f.render_widget(input_nats_server, left_chunk[0]);

                f.set_cursor(
                    // Put cursor past the end of the input text
                    left_chunk[0].x + self.nats_server.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    left_chunk[0].y + 1,
                );

                //let left_up_chunk = Block::default().title("Right Chunk").borders(Borders::ALL);
                // f.render_widget(left_up_chunk, left_chunk[0]);

                // right chunk
                let right_chunk = Block::default().title("Right Chunk").borders(Borders::ALL);
                f.render_widget(right_chunk, chunks[1]);
            })?;

            if let InputEvent::Input(input) = events.next()? {
                if let Event::Key(KeyEvent { code, .. }) = input {
                    match self.input_mode {
                        InputMode::Normal => match code {
                            KeyCode::Enter => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Esc => {
                                break;
                            }
                            _ => {}
                        },
                        InputMode::Editing => match code {
                            KeyCode::Enter => {
                                self.input_mode = InputMode::Normal;
                            }
                            KeyCode::Char(c) => {
                                self.nats_server.push(c);
                            }
                            KeyCode::Backspace => {
                                self.nats_server.pop();
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        Ok(())
    }
}

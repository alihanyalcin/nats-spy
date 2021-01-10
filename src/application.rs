use crate::events::{Events, InputEvent};
use crate::keys::KeyConfig;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct Application {
    left_chunk: Vec<Rect>,
    nats_server: String,
    test_1: String,
    test_2: String,
    input_index: u16,
    input_mode: InputMode,
}

impl Application {
    pub fn new() -> Self {
        Self {
            left_chunk: Vec::new(),
            nats_server: String::new(),
            test_1: String::new(),
            test_2: String::new(),
            input_index: 0,
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
                    .constraints(
                        [
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Length(3),
                            Constraint::Percentage(50),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[0]);

                self.left_chunk = left_chunk.clone();

                let input_nats_server = Paragraph::new(self.nats_server.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("NATS Server"));

                let input_test_1 = Paragraph::new(self.test_1.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("TEST 1"));

                let input_test_2 = Paragraph::new(self.test_2.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("TEST 2"));

                // render left chunk widgets
                f.render_widget(input_nats_server, left_chunk[0]);
                f.render_widget(input_test_1, left_chunk[1]);
                f.render_widget(input_test_2, left_chunk[2]);

                // right chunk
                let right_chunk = Block::default().title("Right Chunk").borders(Borders::ALL);
                f.render_widget(right_chunk, chunks[1]);

                match self.input_mode {
                    InputMode::Normal => {}
                    InputMode::Editing => self.set_cursor(f),
                }
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
                                self.get_input().push(c);
                            }
                            KeyCode::Backspace => {
                                self.get_input().pop();
                            }
                            KeyCode::Tab => {
                                self.input_index = (self.input_index + 1) % 3;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        Ok(())
    }

    fn set_cursor<B: Backend>(&mut self, f: &mut Frame<B>) {
        f.set_cursor(
            self.left_chunk[self.input_index as usize].x + self.get_input().width() as u16 + 1,
            self.left_chunk[self.input_index as usize].y + 1,
        );
    }

    fn get_input(&mut self) -> &mut String {
        match self.input_index {
            0 => &mut self.nats_server,
            1 => &mut self.test_1,
            2 => &mut self.test_2,
            _ => &mut self.nats_server,
        }
    }
}

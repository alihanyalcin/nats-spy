use crate::events::{Events, InputEvent};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct Application {
    left_chunk: Vec<Rect>,
    input_nats_url: String,
    input_subject: String,
    input_test_2: String,
    input_index: u16,
    input_mode: InputMode,
    logs: Vec<String>,
}

impl Application {
    pub fn new(nats_url: &str, subject: &str) -> Self {
        Self {
            left_chunk: Vec::new(),
            input_nats_url: nats_url.to_string(),
            input_subject: subject.to_string(),
            input_test_2: String::new(),
            input_index: 0,
            input_mode: InputMode::Normal,
            logs: Vec::new(),
        }
    }

    pub fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.clear()?;

        let mut events = Events::new(self.input_nats_url.clone(), self.input_subject.clone());

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

                let input_nats_server = Paragraph::new(self.input_nats_url.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("NATS Server"));

                let input_test_1 = Paragraph::new(self.input_subject.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("Subject"));

                let input_test_2 = Paragraph::new(self.input_test_2.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("TEST 2"));

                let logs: Vec<ListItem> = self
                    .logs
                    .iter()
                    .enumerate()
                    .map(|(_i, m)| {
                        let content = vec![Spans::from(Span::raw(m))];
                        ListItem::new(content)
                    })
                    .collect();

                let logs =
                    List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));

                // render left chunk widgets
                f.render_widget(input_nats_server, left_chunk[0]);
                f.render_widget(input_test_1, left_chunk[1]);
                f.render_widget(input_test_2, left_chunk[2]);
                f.render_widget(logs, left_chunk[3]);

                // right chunk
                let right_chunk = Block::default().title("Right Chunk").borders(Borders::ALL);
                f.render_widget(right_chunk, chunks[1]);

                match self.input_mode {
                    InputMode::Normal => {}
                    InputMode::Editing => self.set_cursor(f),
                }
            })?;

            // handle events
            match events.next()? {
                InputEvent::Input(input) => {
                    if let Event::Key(KeyEvent { code, .. }) = input {
                        match self.input_mode {
                            InputMode::Normal => match code {
                                KeyCode::Enter => {
                                    self.input_mode = InputMode::Editing;
                                }
                                KeyCode::Esc => {
                                    events.drain();
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
                InputEvent::Logs(log) => self.logs.push(log),
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
            0 => &mut self.input_nats_url,
            1 => &mut self.input_subject,
            2 => &mut self.input_test_2,
            _ => &mut self.input_nats_url,
        }
    }
}

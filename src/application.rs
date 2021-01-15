use crate::events::{Events, InputEvent};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use tui_logger::TuiLoggerWidget;
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

pub struct Application {
    left_chunk: Vec<Rect>,
    input_nats_url: String,
    input_sub_subject: String,
    input_pub_subject: String,
    input_pub_message: String,
    input_username: Option<String>,
    input_password: Option<String>,
    input_token: Option<String>,
    input_credentials: Option<String>,
    input_index: u16,
    input_mode: InputMode,
    messages: Vec<String>,
}

impl Application {
    pub fn new(
        nats_url: String,
        subject: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
        credentials: Option<String>,
    ) -> Self {
        Self {
            left_chunk: Vec::new(),
            input_nats_url: nats_url,
            input_sub_subject: subject,
            input_pub_subject: String::new(),
            input_pub_message: String::new(),
            input_username: username,
            input_password: password,
            input_token: token,
            input_credentials: credentials,
            input_index: 0,
            input_mode: InputMode::Normal,
            messages: Vec::new(),
        }
    }

    pub fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.clear()?;

        let mut events = Events::new(
            self.input_nats_url.clone(),
            self.input_sub_subject.clone(),
            self.input_username.clone(),
            self.input_password.clone(),
            self.input_token.clone(),
            self.input_credentials.clone(),
        );

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
                            Constraint::Length(3),
                            Constraint::Percentage(50),
                        ]
                        .as_ref(),
                    )
                    .split(chunks[0]);

                self.left_chunk = left_chunk.clone();

                let input_nats_server = Paragraph::new(self.input_nats_url.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("NATS Server"));

                let input_subject = Paragraph::new(self.input_sub_subject.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("Subject"));

                let input_pub_subject = Paragraph::new(self.input_pub_subject.as_ref())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Puslish Subject"),
                    )
                    .style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    );

                let input_pub_message = Paragraph::new(self.input_pub_message.as_ref())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Publish Message"),
                    )
                    .style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    );

                let logs: TuiLoggerWidget = TuiLoggerWidget::default()
                    .block(
                        Block::default()
                            .title("Logs")
                            .border_style(Style::default().fg(Color::White).bg(Color::Black))
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White).bg(Color::Black));

                // render left chunk widgets
                f.render_widget(input_nats_server, left_chunk[0]);
                f.render_widget(input_subject, left_chunk[1]);
                f.render_widget(input_pub_subject, left_chunk[2]);
                f.render_widget(input_pub_message, left_chunk[3]);
                f.render_widget(logs, left_chunk[4]);

                // right chunk
                let messages = self
                    .messages
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(i, m)| {
                        Spans::from(vec![
                            Span::styled(
                                format!("[#{}]:", i),
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(format!("{}", m)),
                        ])
                    })
                    .collect::<Vec<_>>();

                let messages = Paragraph::new(messages)
                    .block(Block::default().borders(Borders::ALL).title(Span::styled(
                        "Messages",
                        Style::default().add_modifier(Modifier::BOLD),
                    )))
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false });

                f.render_widget(messages, chunks[1]);

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
                                KeyCode::Char('p') => events.publish(
                                    self.input_pub_subject.clone(),
                                    self.input_pub_message.clone(),
                                ),
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
                                    self.input_index = (self.input_index + 1) % 4;
                                }
                                _ => {}
                            },
                        }
                    }
                }
                InputEvent::Messages(msg) => self.messages.push(msg),
                InputEvent::Tick => continue,
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
            2 => &mut self.input_pub_subject,
            3 => &mut self.input_pub_message,
            _ => {
                self.input_index = 2;
                &mut self.input_pub_subject
            }
        }
    }
}

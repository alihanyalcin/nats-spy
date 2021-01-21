use crate::events::{Events, InputEvent};
use anyhow::Result;
use chrono::{offset::Local, Timelike};
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
    input_nats_url: String,
    input_sub_subject: String,
    input_pub_subject: String,
    input_pub_message: String,
    input_req_subject: String,
    input_req_message: String,
    input_username: Option<String>,
    input_password: Option<String>,
    input_token: Option<String>,
    input_credentials: Option<String>,
    input_index: u16,
    input_mode: InputMode,
    messages: Vec<(String, String)>,
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
            input_nats_url: nats_url,
            input_sub_subject: subject,
            input_pub_subject: String::new(),
            input_pub_message: String::new(),
            input_req_subject: String::new(),
            input_req_message: String::new(),
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

                self.draw_left_chunk(chunks[0], f);
                self.draw_right_chunk(chunks[1], f);
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
                                KeyCode::Char('q') => {
                                    events.drain();
                                    break;
                                }
                                KeyCode::Char('p') => events.publish(
                                    self.input_pub_subject.clone(),
                                    self.input_pub_message.clone(),
                                ),
                                KeyCode::Char('r') => events.request(
                                    self.input_req_subject.clone(),
                                    self.input_req_message.clone(),
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
                                    self.input_index = (self.input_index + 1) % 6;
                                }
                                _ => {}
                            },
                        }
                    }
                }
                InputEvent::Messages(topic, msg) => self.messages.push((topic, msg)),
                InputEvent::Tick => {}
            }
        }

        Ok(())
    }

    fn draw_right_chunk<B: Backend>(&self, chunk: Rect, f: &mut Frame<B>) {
        // nats messages
        let messages = self
            .messages
            .iter()
            .enumerate()
            .rev()
            .map(|(i, (t, m))| {
                Spans::from(vec![
                    Span::styled(
                        format!("[#{}] ", i),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("[{}]: ", t),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("{}", m)),
                ])
            })
            .collect::<Vec<_>>();

        let messages = Paragraph::new(messages)
            .block(Block::default().borders(Borders::ALL).title(Span::styled(
                format!("Messages - {}", self.get_time()),
                Style::default().add_modifier(Modifier::BOLD),
            )))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        f.render_widget(messages, chunk);
    }

    fn draw_left_chunk<B: Backend>(&mut self, chunk: Rect, f: &mut Frame<B>) {
        // left chunk
        let left_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Percentage(30),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(chunk);

        // nats server url
        let input_nats_server = Paragraph::new(self.input_nats_url.as_ref())
            .block(Block::default().borders(Borders::ALL).title("NATS Server"));

        // nats subscription subject
        let input_subject = Paragraph::new(self.input_sub_subject.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Subject"));

        // nats publish subject
        let input_pub_subject = Paragraph::new(self.input_pub_subject.as_ref())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Publish Subject"),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        // nats puslish message
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

        // nats request subject
        let input_req_subject = Paragraph::new(self.input_req_subject.as_ref())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Request Subject"),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        // nats request message
        let input_req_message = Paragraph::new(self.input_req_message.as_ref())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Request Message"),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        // log widget
        let logs: TuiLoggerWidget =
            TuiLoggerWidget::default().block(Block::default().title("Logs").borders(Borders::ALL));

        // help message
        let help = match self.input_mode {
            InputMode::Normal => {
                vec![
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled(
                            "Q",
                            Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
                        ),
                        Span::raw(" to exit, "),
                        Span::styled(
                            "ENTER",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Blue),
                        ),
                        Span::raw(" to start editing."),
                    ]),
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled(
                            "P",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Cyan),
                        ),
                        Span::raw(" to publish, "),
                        Span::styled(
                            "R",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::LightCyan),
                        ),
                        Span::raw(" to request."),
                    ]),
                ]
            }
            InputMode::Editing => {
                vec![
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled(
                            "ENTER",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Blue),
                        ),
                        Span::raw(" to stop editing."),
                    ]),
                    Spans::from(vec![
                        Span::raw("Press "),
                        Span::styled(
                            "TAB",
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Magenta),
                        ),
                        Span::raw(" to move cursor."),
                    ]),
                ]
            }
        };

        let help_message =
            Paragraph::new(help).block(Block::default().borders(Borders::ALL).title("Help"));

        // render left chunk widgets
        f.render_widget(input_nats_server, left_chunk[0]);
        f.render_widget(input_subject, left_chunk[1]);
        f.render_widget(input_pub_subject, left_chunk[2]);
        f.render_widget(input_pub_message, left_chunk[3]);
        f.render_widget(input_req_subject, left_chunk[4]);
        f.render_widget(input_req_message, left_chunk[5]);
        f.render_widget(logs, left_chunk[6]);
        f.render_widget(help_message, left_chunk[7]);

        // set cursor for editing mode
        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Editing => self.set_cursor(left_chunk, f),
        }
    }

    fn set_cursor<B: Backend>(&mut self, chunk: Vec<Rect>, f: &mut Frame<B>) {
        f.set_cursor(
            chunk[self.input_index as usize].x + self.get_input().width() as u16 + 1,
            chunk[self.input_index as usize].y + 1,
        );
    }

    fn get_input(&mut self) -> &mut String {
        match self.input_index {
            2 => &mut self.input_pub_subject,
            3 => &mut self.input_pub_message,
            4 => &mut self.input_req_subject,
            5 => &mut self.input_req_message,
            _ => {
                self.input_index = 2;
                &mut self.input_pub_subject
            }
        }
    }

    fn get_time(&self) -> String {
        let now = Local::now();

        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
    }
}

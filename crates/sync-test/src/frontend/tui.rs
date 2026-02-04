use std::time::Duration;

use clap::Parser;
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::simulation::{
    config::SyncTestConfig,
    messages::{MessageQueue, MessageToClient, MessageToServer},
    sim::Sim,
    world::{CellLocation, WorldCells},
};

#[derive(Debug, Parser)]
pub struct TuiArgs {
    #[command(flatten)]
    pub config: SyncTestConfig,
    // TODO: add sub-set of Tui struct
}

#[derive(Debug)]
pub struct Tui {
    exit: bool,
    sim: Sim,
    config: SyncTestConfig,
    auto_tick: bool,
    auto_tick_duration: Duration,
    manual_tick: bool,
    reset: bool,
}

impl Tui {
    pub fn new(args: TuiArgs) -> Self {
        Self {
            exit: false,
            sim: Sim::new(args.config.clone()),
            config: args.config,
            auto_tick: false,
            auto_tick_duration: Duration::from_millis(10),
            manual_tick: false,
            reset: false,
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            self.manual_tick = false;
            self.reset = false;

            if self.auto_tick {
                if event::poll(self.auto_tick_duration)? {
                    self.handle_events()?;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if event::poll(Duration::from_millis(100))? {
                    self.handle_events()?;
                }
            }

            if self.auto_tick || self.manual_tick {
                self.sim.tick();
            }

            if self.reset {
                self.sim = Sim::new(self.config.clone());
            }

            term.draw(|frame| frame.render_widget(&*self, frame.area()))?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            event::Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('c')
                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.exit = true;
            }
            KeyCode::Char(' ') => {
                self.manual_tick = true;
            }
            KeyCode::Char('t') => {
                self.auto_tick = !self.auto_tick;
            }
            KeyCode::Char('r') => {
                self.reset = true;
            }
            _ => {}
        }
    }
}

impl Widget for &Tui {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // TODO: just pass around immutable reference to self?

        let top_bar = TopBar {
            auto_tick: &self.auto_tick,
            manual_tick: &self.manual_tick,
            reset: &self.reset,
            tick: &self.sim.tick.0,
        };

        let server_world = SimpleWorldCellsWidget {
            name: "server",
            world: &self.sim.server_world,
        };
        let client_world = SimpleWorldCellsWidget {
            name: "client",
            world: &self.sim.client_world,
        };

        let server_queue = MessageQueueWidget {
            name: "server",
            queue: &self.sim.server_queue,
        };
        let client_queue = MessageQueueWidget {
            name: "client",
            queue: &self.sim.client_queue,
        };

        let top_level_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(top_bar.num_lines()),
                Constraint::Fill(2),
            ]);

        let horizontal_50_50 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);

        let vertical_70_30 = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)]);

        let [top_area, main_area] = top_level_layout.areas(area);
        let [server_area, client_area] = horizontal_50_50.areas(main_area);

        let [server_world_area, server_queue_area] = vertical_70_30.areas(server_area);
        let [client_world_area, client_queue_area] = vertical_70_30.areas(client_area);

        top_bar.render(top_area, buf);
        client_world.render(client_world_area, buf);
        server_world.render(server_world_area, buf);
        client_queue.render(client_queue_area, buf);
        server_queue.render(server_queue_area, buf);
    }
}

struct TopBar<'a> {
    auto_tick: &'a bool,
    manual_tick: &'a bool,
    reset: &'a bool,
    tick: &'a u128,
}

impl TopBar<'_> {
    const fn num_lines(&self) -> u16 {
        // TODO: const generate lines template to render?
        3
    }
}

impl<'a> Widget for TopBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" sync-test TUI ").bold();

        let auto_tick = " Auto-tick ".fg(if *self.auto_tick {
            Color::Green
        } else {
            Color::Red
        });

        let manual_tick = " Manual tick ".fg(if *self.manual_tick {
            Color::Green
        } else {
            Color::default()
        });

        let reset = " Reset ".fg(if *self.reset {
            Color::Green
        } else {
            Color::default()
        });

        let keybinds = Line::from(vec![
            " Quit ".into(),
            "<Ctrl+C/Ctrl+Q>".blue().bold(),
            auto_tick,
            "<T>".blue().bold(),
            manual_tick,
            "<Space>".blue().bold(),
            reset,
            "<R>".blue().bold(),
        ]);

        let tick_number = Line::from(vec![" Tick: ".into(), self.tick.to_string().red()]);

        let para =
            Paragraph::new(Text::from(vec![title, keybinds, tick_number]).centered()).centered();

        para.render(area, buf);
    }
}

struct SimpleWorldCellsWidget<'a> {
    name: &'a str,
    world: &'a WorldCells,
}

impl<'a> Widget for SimpleWorldCellsWidget<'a> {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let para = Paragraph::new(Text::from(self.world.to_string()).centered())
            .centered()
            .block(Block::bordered().title(Line::from(format!(" {} ", self.name)).centered()));

        para.render(area, buf);
    }
}

struct MessageQueueWidget<'a, Message>
where
    Vec<Span<'static>>: From<&'a Message>,
{
    name: &'static str,
    queue: &'a MessageQueue<Message>,
}

impl<'a, Message> Widget for MessageQueueWidget<'a, Message>
where
    Vec<Span<'static>>: From<&'a Message>,
{
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let lines = self
            .queue
            .iter()
            .map(|message| {
                let mut spans = vec![
                    "@".into(),
                    format!("{:04}", message.tick_to_arrive.0).blue(),
                    " ".into(),
                ];
                let inner_message_spans: Vec<Span<'static>> = (&message.message).into();
                spans.extend(inner_message_spans);
                Line::from(spans)
            })
            .collect::<Vec<_>>();

        let para = Paragraph::new(Text::from(lines)).block(
            Block::bordered()
                .title(Line::from(format!(" packets in flight to {}", self.name)).centered()),
        );

        para.render(area, buf);
    }
}

// TODO: replace with iterators to avoid allocs

impl From<&CellLocation> for Vec<Span<'static>> {
    fn from(location: &CellLocation) -> Self {
        vec![
            "(".into(),
            format!("{:02}", location.row).blue(),
            ",".into(),
            format!("{:02}", location.column).blue(),
            ")".into(),
        ]
    }
}

impl From<&MessageToServer> for Vec<Span<'static>> {
    fn from(message: &MessageToServer) -> Self {
        let ack = ["ack".into(), message.ack.0.to_string().blue(), " ".into()];

        let cells = message.cells.iter().map(Into::<Vec<Span<'static>>>::into);

        ack.into_iter().chain(cells.flatten()).collect::<Vec<_>>()
    }
}

impl From<&MessageToClient> for Vec<Span<'static>> {
    fn from(message: &MessageToClient) -> Self {
        let tick = [
            "tick".into(),
            format!("{:04}", message.tick.0).red(),
            " ".into(),
        ];

        let updates = message.updates.iter().map(|update| {
            let location = Into::<Vec<Span<'static>>>::into(&update.id);
            let state = format!("{:02x}", update.new_state).magenta();
            let priority = format!("{:.2}", update.priority).yellow();

            let mut spans = vec![];
            spans.extend(location);
            spans.push("=".into());
            spans.push(state);
            spans.extend(["p".into(), priority, " ".into()]);
            spans
        });

        tick.into_iter()
            .chain(updates.flatten())
            .collect::<Vec<_>>()
    }
}

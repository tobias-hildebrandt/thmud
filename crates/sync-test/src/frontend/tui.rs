use std::{str::FromStr, time::Duration};

use bpaf::Bpaf;
use colorgrad::{Gradient, GradientBuilder, LinearGradient};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    simulation::{
        config::SimConfig,
        messages::{MessageQueue, MessageToClient, MessageToServer},
        priority::WorldPriority,
        sim::{Sim, Tick},
        sync::{InFlightSyncStatus, WorldInFlightSyncStatus},
        world::{CellLocation, WorldCells},
    },
    utils::square,
};

#[derive(Debug, Clone, Bpaf)]
#[bpaf(group_help("TUI options:"), generate(tui_config_parser))]
pub struct TuiConfig {
    /// Run the simulation on startup
    #[bpaf(
        switch,
        long("start"),
        fallback(TuiConfig::default_auto_tick()),
        display_fallback
    )]
    pub auto_tick: bool,
    /// Auto-tick delay (in millis)
    #[bpaf(long("tick"), fallback(TuiConfig::default_auto_tick_duration().as_millis() as u64), display_fallback,
    argument::<u64>, map(Duration::from_millis))]
    pub auto_tick_duration: Duration,
}

impl TuiConfig {
    fn default_auto_tick() -> bool {
        false
    }

    fn default_auto_tick_duration() -> Duration {
        Duration::from_millis(10)
    }
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            auto_tick: Self::default_auto_tick(),
            auto_tick_duration: Self::default_auto_tick_duration(),
        }
    }
}

#[derive(Debug)]
pub struct Tui {
    sim: Sim,
    inputs: TuiInputs,
    tui_config: TuiConfig,
}

#[derive(Debug, Default)]
struct TuiInputs {
    exit: bool,
    manual_tick: bool,
    reset: bool,
}

impl Tui {
    pub fn new(tui_config: TuiConfig, sim_config: SimConfig) -> Self {
        Self {
            sim: Sim::new(sim_config),
            tui_config,
            inputs: TuiInputs::default(),
        }
    }

    /// Run the TUI.
    ///
    /// In general, this follows a classic read-evaluate-print loop.
    pub fn run(&mut self, term: &mut DefaultTerminal) -> std::io::Result<()> {
        // print first thing so we have something on the screen immediately
        term.draw(|frame| frame.render_widget(&*self, frame.area()))?;

        while !self.inputs.exit {
            // reset button presses
            self.inputs = Default::default();

            // read
            if self.tui_config.auto_tick {
                // poll for events for the auto tick duration
                if event::poll(self.tui_config.auto_tick_duration)? {
                    self.handle_events()?;
                }
            } else {
                // poll for events for a reasonable duration
                #[allow(clippy::collapsible_else_if)]
                if event::poll(Duration::from_millis(100))? {
                    self.handle_events()?;
                }
            }

            // evaluate
            if self.tui_config.auto_tick || self.inputs.manual_tick {
                self.sim.tick();
            }

            if self.inputs.manual_tick {
                self.tui_config.auto_tick = false;
            }

            if self.inputs.reset {
                self.sim = Sim::new(self.sim.config.clone());
            }

            // print
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
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.inputs.exit = true;
            }
            KeyCode::Char('q') => {
                self.inputs.exit = true;
            }
            KeyCode::Char(' ') => {
                self.inputs.manual_tick = true;
            }
            KeyCode::Char('t') => {
                self.tui_config.auto_tick = !self.tui_config.auto_tick;
            }
            KeyCode::Char('r') => {
                self.inputs.reset = true;
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
            auto_tick: &self.tui_config.auto_tick,
            manual_tick: &self.inputs.manual_tick,
            reset: &self.inputs.reset,
            tick: &self.sim.tick.0,
        };

        let server_world = WorldCellPriorityWidget {
            name: "server",
            world: &self.sim.server.world,
            priorities: &self.sim.server.priorities,
        };
        let client_world = WorldCellSyncWidget {
            name: "client",
            world: &self.sim.client.world,
            sync: &self.sim.sync_status,
        };

        let server_queue = MessageQueueWidget {
            name: "server",
            queue: &self.sim.server.message_queue,
        };
        let client_queue = MessageQueueWidget {
            name: "client",
            queue: &self.sim.client.message_queue,
        };

        let priority = PriorityListWidget {
            priority: &self.sim.server.priorities,
        };

        let top_level_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(top_bar.num_lines()),
                Constraint::Fill(2),
            ]);

        let horizontal_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);

        let vertical_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(70),
                Constraint::Length(1),
                Constraint::Percentage(30),
            ]);

        let [top_area, main_area] = top_level_layout.areas(area);

        let [world_area, priority_area, queue_area] = vertical_split.areas(main_area);

        let [server_world_area, client_world_area] = horizontal_split.areas(world_area);
        let [server_queue_area, client_queue_area] = horizontal_split.areas(queue_area);

        top_bar.render(top_area, buf);
        client_world.render(client_world_area, buf);
        server_world.render(server_world_area, buf);
        client_queue.render(client_queue_area, buf);
        server_queue.render(server_queue_area, buf);
        priority.render(priority_area, buf);
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
            "<Q/Ctrl+C>".blue().bold(),
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

struct WorldCellSyncWidget<'a> {
    name: &'a str,
    world: &'a WorldCells,
    sync: &'a WorldInFlightSyncStatus,
}

impl<'a> Widget for WorldCellSyncWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut text = Text::default();
        let world_size = self.world.world_size();
        for row in 0..world_size {
            let mut line = Line::default();
            for column in 0..world_size {
                let value = self.world.data[row][column].state;
                let sync = &self.sync.data[row][column];
                let sync_color = Into::<Color>::into(sync);

                let span = format!("{:02x}", value).fg(sync_color);
                line.push_span(span);
            }
            text.push_line(line);
        }

        let para = Paragraph::new(text.centered()).centered().block(
            Block::bordered()
                .title(Line::from(format!(" {} (sync status) ", self.name)).centered()),
        );

        para.render(area, buf);
    }
}

struct WorldCellPriorityWidget<'a> {
    name: &'a str,
    world: &'a WorldCells,
    priorities: &'a WorldPriority,
}

impl<'a> Widget for WorldCellPriorityWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        //TODO: move up
        let priority_curve = GradientBuilder::new()
            .colors(&[
                colorgrad::Color::from_rgba8(255, 0, 0, 255),
                colorgrad::Color::from_rgba8(0, 255, 0, 255),
            ])
            .build::<LinearGradient>()
            .expect("unable to build priority color gradient");

        let max_priority = self
            .priorities
            .data
            .iter()
            .flatten()
            .map(|p| p.0)
            .sum::<f32>()
            / (square(self.priorities.world_size()) as f32);

        let mut text = Text::default();
        let world_size = self.world.world_size();
        for row in 0..world_size {
            let mut line = Line::default();
            for column in 0..world_size {
                let value = self.world.data[row][column].state;
                let priority = &self.priorities.data[row][column];

                let color = if max_priority == 0.0 {
                    Color::Gray
                } else {
                    let relative_priority = priority.0 / max_priority;
                    let colorgrad_color = priority_curve.at(relative_priority).to_css_hex();
                    Color::from_str(&colorgrad_color)
                        .expect("conversion between colorgrad and ratataui colors failed")
                };

                let span = format!("{:02x}", value).fg(color);
                line.push_span(span);
            }
            text.push_line(line);
        }

        let para = Paragraph::new(text.centered()).centered().block(
            Block::bordered().title(Line::from(format!(" {} (priority) ", self.name)).centered()),
        );

        para.render(area, buf);
    }
}

struct PriorityListWidget<'a> {
    priority: &'a WorldPriority,
}

impl<'a> Widget for PriorityListWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut priorities = self.priority.cell_priorities().collect::<Vec<_>>();

        priorities.sort_by(|first, second| second.priority.0.total_cmp(&first.priority.0));

        let spans = priorities
            .into_iter()
            .flat_map(|priority| {
                let location = Into::<Vec<Span<'static>>>::into(&priority.location);
                let priority = format!("{:.2}", priority.priority.0).yellow();

                let mut spans = vec![];
                spans.extend(location);
                spans.push(priority);
                spans.push(" ".into());
                spans
            })
            .collect::<Vec<_>>();

        let para = Paragraph::new(Text::from(Line::from(spans)));

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
                let mut spans = vec!["@".into(), message.tick_to_arrive.into(), " ".into()];
                let inner_message_spans: Vec<Span<'static>> = (&message.message).into();
                spans.extend(inner_message_spans);
                Line::from(spans)
            })
            .collect::<Vec<_>>();

        let para = Paragraph::new(Text::from(lines)).block(
            Block::bordered()
                .title(Line::from(format!(" packets in flight to {} ", self.name)).centered()),
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

impl From<Tick> for Span<'static> {
    fn from(tick: Tick) -> Self {
        format!("{:04}", tick.0).blue()
    }
}

impl From<&MessageToServer> for Vec<Span<'static>> {
    fn from(message: &MessageToServer) -> Self {
        let ack = [
            "ack".into(),
            Into::<Span>::into(message.ack).red(),
            " ".into(),
        ];

        let cells = message.cells.iter().map(Into::<Vec<Span<'static>>>::into);

        ack.into_iter().chain(cells.flatten()).collect::<Vec<_>>()
    }
}

impl From<&MessageToClient> for Vec<Span<'static>> {
    fn from(message: &MessageToClient) -> Self {
        let tick = [
            "tick".into(),
            Into::<Span>::into(message.tick).red(),
            " ".into(),
        ];

        let updates = message.updates.iter().map(|update| {
            let location = Into::<Vec<Span<'static>>>::into(&update.id);
            let state = format!("{:02x}", update.new_state).magenta();
            let priority = format!("{:.2}", update.priority.0).yellow();

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

impl From<&InFlightSyncStatus> for Color {
    fn from(value: &InFlightSyncStatus) -> Self {
        match (
            value.same,
            value.update_in_flight.old,
            value.update_in_flight.current,
        ) {
            // up to date, nothing in flight
            (true, false, false) => Color::Green,
            // up to date, useless copy in flight
            (true, false, true) => Color::LightGreen,
            // up to date, but old in flight ??
            (true, true, false) => Color::Yellow,
            // up to date but both old and new in flight??
            (true, true, true) => Color::Magenta,
            // not up to date but current update in flight
            (false, _, true) => Color::LightYellow,
            // not update to date but old in flight
            (false, true, false) => Color::Blue,
            // not synced
            (false, false, false) => Color::Red,
        }
    }
}

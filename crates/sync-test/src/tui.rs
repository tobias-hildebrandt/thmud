use std::time::Duration;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, canvas::Canvas},
};

use crate::{config::SyncTestConfig, sim::Sim};

#[derive(Debug)]
pub struct Tui {
    exit: bool,
    sim: Sim,
    auto_tick: bool,
    auto_tick_duration: Duration,
}

impl Tui {
    pub fn new(config: SyncTestConfig) -> Self {
        Self {
            exit: false,
            sim: Sim::new(config),
            auto_tick: false,
            auto_tick_duration: Duration::from_millis(10),
        }
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> std::io::Result<()> {
        while !self.exit {
            term.draw(|frame| self.draw(frame))?;
            if self.auto_tick {
                if event::poll(self.auto_tick_duration)? {
                    self.handle_events()?;
                }
                self.sim.tick();
            } else {
                #[allow(clippy::collapsible_else_if)]
                if event::poll(Duration::from_millis(100))? {
                    self.handle_events()?;
                }
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let title = Line::from(" sync-test TUI ").bold();

        const AUTO_TICK_TEXT: &str = " Auto-tick ";
        let keybinds = Line::from(vec![
            " Quit ".into(),
            "<Ctrl+C/Ctrl+Q>".blue().bold(),
            if self.auto_tick {
                AUTO_TICK_TEXT.green()
            } else {
                AUTO_TICK_TEXT.red()
            },
            "<T>".blue().bold(),
            " Manual tick ".into(),
            "<Space>".blue().bold(),
        ]);

        let top_text = Text::from(vec![title, keybinds]).centered();
        let paragraph = Paragraph::new(top_text).centered();

        let server_world =
            Paragraph::new(Text::from(format!("{}", self.sim.server_world)).centered())
                .centered()
                .block(Block::bordered().title(Line::from("server").centered()));
        let client_world =
            Paragraph::new(Text::from(format!("{}", self.sim.client_world)).centered())
                .centered()
                .block(Block::bordered().title(Line::from("client").centered()));

        let top_level_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(2), Constraint::Fill(2)]);

        let screens_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)]);

        let [top_bar, main_screen] = top_level_layout.areas(frame.area());
        let [server_screen, client_screen] = screens_layout.areas(main_screen);

        frame.render_widget(paragraph, top_bar);
        frame.render_widget(client_world, client_screen);
        frame.render_widget(server_world, server_screen);
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
                self.sim.tick();
            }
            KeyCode::Char('t') => {
                self.auto_tick = !self.auto_tick;
            }

            _ => {}
        }
    }
}

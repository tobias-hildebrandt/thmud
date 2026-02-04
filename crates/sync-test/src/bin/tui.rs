use clap::Parser;
use sync_test::{config::SyncTestConfig, tui::Tui};

fn main() -> std::io::Result<()> {
    let config = SyncTestConfig::parse();

    let mut tui = Tui::new(config);

    println!("entering tui");

    ratatui::run(|terminal| tui.run(terminal))?;

    println!("exited tui");

    Ok(())
}

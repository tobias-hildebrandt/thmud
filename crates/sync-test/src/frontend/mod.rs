pub mod args;
pub mod simple_cli;
pub mod tui;

/// Parse program arguments and run the frontend.
pub fn parse_args_and_run() {
    let args = <args::Arguments as clap::Parser>::parse();

    args.frontend.run();
}

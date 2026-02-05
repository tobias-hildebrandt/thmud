pub mod args;
pub mod simple_cli;
pub mod tui;

/// Parses program arguments and runs the frontend.
pub fn parse_args_and_run() {
    let args = args::arguments_parser().run();

    args.frontend.run(args.sim_config)
}

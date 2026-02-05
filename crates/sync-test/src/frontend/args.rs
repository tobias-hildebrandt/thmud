use bpaf::Bpaf;

use crate::{
    frontend::{
        simple_cli::{SimpleCliConfig, run_simple_cli, simple_cli_config_parser},
        tui::{Tui, TuiConfig, tui_config_parser},
    },
    simulation::config::{SimConfig, config_parser},
};

/// Creates the parser for [`Arguments`].
pub fn arguments_parser() -> bpaf::OptionParser<Arguments> {
    arguments_parser_private().help_parser(
        bpaf::long("help")
            .short('h')
            .help("Print usage and help information"),
    )
}

/// Overall program arguments
#[derive(Debug, Clone, Bpaf)]
#[bpaf(
    options,
    descr("Sync-test, a netcode tester"),
    generate(arguments_parser_private),
    private
)]
pub struct Arguments {
    #[bpaf(external(frontend_arguments_parser))]
    pub frontend: Frontend,

    #[bpaf(external(config_parser))]
    pub sim_config: SimConfig,
}

/// Frontend options
#[derive(Debug, Clone, Bpaf)]
#[bpaf(ignore_rustdoc, generate(frontend_arguments_parser))]
pub enum Frontend {
    SimpleCli(#[bpaf(external(cli_frontend_config_parser))] CliFrontendConfig),
    Tui(#[bpaf(external(tui_frontend_config_parser))] TuiFrontendConfig),
}

impl Frontend {
    /// Runs the frontend.
    pub fn run(self, sim_config: SimConfig) {
        match self {
            Frontend::SimpleCli(CliFrontendConfig { cli_config, .. }) => {
                run_simple_cli(cli_config, sim_config);
            }
            Frontend::Tui(TuiFrontendConfig { tui_config, .. }) => {
                ratatui::run(|terminal| Tui::new(tui_config, sim_config).run(terminal)).unwrap();
            }
        }
    }
}

#[derive(Debug, Clone, Bpaf)]
#[allow(clippy::manual_non_exhaustive)]
#[bpaf(generate(cli_frontend_config_parser), group_help("CLI options:"))]
pub struct CliFrontendConfig {
    /// Use the CLI frontend
    #[bpaf(long("cli"))]
    _cli: (),
    #[bpaf(external(simple_cli_config_parser))]
    pub cli_config: SimpleCliConfig,
}

impl From<SimpleCliConfig> for CliFrontendConfig {
    fn from(cli_config: SimpleCliConfig) -> Self {
        Self {
            _cli: (),
            cli_config,
        }
    }
}

#[derive(Debug, Clone, Bpaf)]
#[allow(clippy::manual_non_exhaustive)]
#[bpaf(generate(tui_frontend_config_parser), group_help("TUI options:"))]
pub struct TuiFrontendConfig {
    /// Use the TUI frontend
    #[bpaf(long("tui"), fallback(()))]
    _tui: (),
    #[bpaf(external(tui_config_parser))]
    pub tui_config: TuiConfig,
}

impl From<TuiConfig> for TuiFrontendConfig {
    fn from(tui_config: TuiConfig) -> Self {
        Self {
            _tui: (),
            tui_config,
        }
    }
}

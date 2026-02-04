use clap::Parser;
use sync_test::frontend::args::Arguments;

fn main() {
    let args = Arguments::parse();

    args.frontend.run(args.config);
}

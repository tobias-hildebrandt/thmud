use thmud::{RunType, app};

fn main() {
    let run_type = if std::env::args().any(|a| a == "client") {
        RunType::Client
    } else {
        RunType::Server
    };

    app(run_type).run();
}

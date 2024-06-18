use clap::Parser;
use journalr::{app::App, args::Arguments};

fn main() {
    let app = App::try_from(Arguments::parse());
    match app {
        Ok(app) => app.run().unwrap_or_else(|e| println!("{e}")),
        Err(e) => eprintln!("{e}"),
    }
}

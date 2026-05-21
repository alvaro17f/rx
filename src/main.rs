mod app;
mod core;
mod error;

use app::cli::RealDeps;
use app::init;
use std::{env, io::stdout};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out = stdout();
    let deps = RealDeps;
    if let Err(e) = init::run(&args, &mut out, &deps) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

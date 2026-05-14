mod app;
mod core;
mod error;

#[cfg(not(test))]
use app::init;
#[cfg(not(test))]
use std::{env, io::stdout};
#[cfg(not(test))]
use app::cli::RealDeps;

#[cfg(not(test))]
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut out = stdout();
    let deps = RealDeps;
    if let Err(e) = init::run(&args, &mut out, &deps) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

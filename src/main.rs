#![feature(coverage_attribute)]

mod app;
mod core;
mod error;

use std::env;
use std::io::stdout;

use app::cli::RealDeps;
use app::init;
use error::Error;

#[coverage(off)]
fn run_main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut out = stdout();
    let deps = RealDeps;
    init::run(&args, &mut out, &deps)
}

#[coverage(off)]
fn main() {
    if let Err(e) = run_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
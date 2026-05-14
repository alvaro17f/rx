mod app;
mod core;
mod error;

use std::env;
use std::io::stdout;

use app::cli::RealDeps;
use app::init;
use error::Error;

/// Collect CLI arguments, wire real dependencies, and run the application.
pub fn run_main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let mut out = stdout();
    let deps = RealDeps;
    init::run(&args, &mut out, &deps)
}

fn main() {
    if let Err(e) = run_main() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app::cli::Deps;
    use app::init::Config;

    struct MockDeps;

    impl Deps for MockDeps {
        fn run_shell(&self, _: &str, _: bool) -> Result<i32, Error> { Ok(0) }
        fn confirm(&self, _: bool, _: Option<&str>) -> Result<bool, Error> { Ok(false) }
        fn print_title(&self, _: &str) -> Result<(), Error> { Ok(()) }
        fn config_print(&self, _: &Config) -> Result<(), Error> { Ok(()) }
    }

    #[test]
    fn run_main_routes_help_flag_to_help_text() {
        let args = vec!["rx".to_string(), "-h".to_string()];
        let mut buf = Vec::new();
        init::run(&args, &mut buf, &MockDeps).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("RX"));
    }

    #[test]
    fn run_main_routes_version_flag_to_version_text() {
        let args = vec!["rx".to_string(), "-v".to_string()];
        let mut buf = Vec::new();
        init::run(&args, &mut buf, &MockDeps).unwrap();
    }

    #[test]
    fn run_main_returns_error_on_unknown_flag() {
        let args = vec!["rx".to_string(), "-x".to_string()];
        let mut buf = Vec::new();
        assert!(init::run(&args, &mut buf, &MockDeps).is_err());
    }

    #[test]
    fn run_main_no_args_reaches_cli_and_returns_ok() {
        let args = vec!["rx".to_string()];
        let mut buf = Vec::new();
        init::run(&args, &mut buf, &MockDeps).unwrap();
    }
}
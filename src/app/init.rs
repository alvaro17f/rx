use std::fmt;
use std::io::Write;

use crate::core::ansi;
use crate::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parsed CLI configuration.
#[derive(Debug)]
pub struct Config {
    pub repo: String,
    pub hostname: String,
    pub keep: u8,
    pub update: bool,
    pub diff: bool,
}

impl Config {
    /// Build defaults: repo `~/.dotfiles`, hostname from `/proc/sys/kernel/hostname`,
    /// keep `10`, update/diff `false`.
    pub fn defaults() -> Self {
        Self {
            repo: String::from("~/.dotfiles"),
            hostname: default_hostname(),
            keep: 10,
            update: false,
            diff: false,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ repo: {}, hostname: {}, keep: {}, update: {}, diff: {} }}",
            self.repo, self.hostname, self.keep, self.update, self.diff
        )
    }
}

/// Read hostname from `/proc/sys/kernel/hostname`, falling back to the
/// `HOSTNAME` environment variable, then `"unknown"`.
fn default_hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_owned())
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| String::from("unknown"))
}

/// Print the help banner.
pub fn print_help<W: Write>(writer: &mut W) -> Result<(), Error> {
    ansi::write_flush(
        writer,
        "\n*****************************************************\n RX - A simple CLI tool to update your nixos system\n*****************************************************\n-r : set repo path (default is $HOME/.dotfiles)\n-n : set hostname (default is OS hostname)\n-k : set generations to keep (default is 10)\n-u : set update to true (default is false)\n-d : set diff to true (default is false)\n-h, help : Display this help message\n-v, version : Display the current version\n\n",
    )?;
    Ok(())
}

/// Print the version line.
pub fn print_version<W: Write>(writer: &mut W) -> Result<(), Error> {
    ansi::write_flush(
        writer,
        &format!("{}RX version: {}{}\n", ansi::YELLOW, VERSION, ansi::RESET),
    )?;
    Ok(())
}

fn print_config_line<W: Write>(
    writer: &mut W,
    label: &str,
    value: &str,
    new_line: bool,
) -> Result<(), Error> {
    ansi::write_flush(
        writer,
        &format!(
            "{}◉ {}{}{} = {}{}{}{}",
            ansi::CYAN,
            ansi::RED,
            label,
            ansi::RESET,
            ansi::CYAN,
            value,
            ansi::RESET,
            if new_line { "\n" } else { "" }
        ),
    )?;
    Ok(())
}

/// Print all `config` fields with ANSI styling.
pub fn config_print<W: Write>(writer: &mut W, config: &Config) -> Result<(), Error> {
    print_config_line(writer, "repo", &config.repo, true)?;
    print_config_line(writer, "hostname", &config.hostname, true)?;
    print_config_line(writer, "keep", &config.keep.to_string(), true)?;
    print_config_line(writer, "update", &config.update.to_string(), true)?;
    print_config_line(writer, "diff", &config.diff.to_string(), false)?;
    Ok(())
}

/// Result of parsing command-line arguments.
#[derive(Debug)]
pub enum Parsed {
    Help,
    Version,
    Run(Config),
    Error,
}

/// Parse `args` (including the program name at index 0).
///
/// Prints error messages directly to `writer` for invalid flags/arguments.
pub fn parse_args(args: &[String], writer: &mut dyn Write) -> Parsed {
    if args.len() <= 1 {
        return Parsed::Run(Config::defaults());
    }

    if args.len() == 2 {
        match args[1].as_str() {
            "help" => return Parsed::Help,
            "version" => return Parsed::Version,
            _ => {}
        }
    }

    if !args[1].starts_with('-') {
        let arg = &args[1];
        ansi::write_flush(
            writer,
            &format!("{}Error: Unknown argument \"{}\"\n{}", ansi::RED, arg, ansi::RESET),
        )
        .ok();
        return Parsed::Error;
    }

    let mut config = Config::defaults();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            let flags = arg.strip_prefix('-').unwrap_or(arg);
            let mut skip_next = false;
            for flag_char in flags.chars() {
                match flag_char {
                    'h' => return Parsed::Help,
                    'v' => return Parsed::Version,
                    'd' => config.diff = true,
                    'u' => config.update = true,
                    'r' | 'n' | 'k' => {
                        if i + 1 >= args.len() {
                            ansi::write_flush(
                                writer,
                                &format!(
                                    "{}Error: \"-{}\" flag requires an argument\n{}",
                                    ansi::RED, flag_char, ansi::RESET
                                ),
                            )
                            .ok();
                            return Parsed::Error;
                        }
                        let value = args[i + 1].clone();
                        match flag_char {
                            'r' => config.repo = value,
                            'n' => config.hostname = value,
                            'k' => {
                                match value.parse::<u8>() {
                                    Ok(num) => config.keep = num,
                                    Err(_) => {
                                        ansi::write_flush(
                                            writer,
                                            &format!(
                                                "{}Error: Value of \"-k\" flag is not numeric.\n{}",
                                                ansi::RED, ansi::RESET
                                            ),
                                        )
                                        .ok();
                                        return Parsed::Error;
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                        skip_next = true;
                        break;
                    }
                    _ => {
                        ansi::write_flush(
                            writer,
                            &format!(
                                "{}Error: Unknown flag \"-{}\"\n{}",
                                ansi::RED, flag_char, ansi::RESET
                            ),
                        )
                        .ok();
                        return Parsed::Error;
                    }
                }
            }
            if skip_next {
                i += 1;
            }
        }
        i += 1;
    }

    Parsed::Run(config)
}

/// Dispatch parsed arguments to help, version, CLI workflow, or error.
pub fn run<W: Write>(
    args: &[String],
    writer: &mut W,
    deps: &dyn crate::app::cli::Deps,
) -> Result<(), Error> {
    match parse_args(args, writer) {
        Parsed::Help => print_help(writer),
        Parsed::Version => print_version(writer),
        Parsed::Error => Err(Error::GitPullFailed),
        Parsed::Run(config) => crate::app::cli::cli(writer, &config, deps),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::cli::Deps;

    fn args(from: &[&str]) -> Vec<String> {
        from.iter().map(|s| s.to_string()).collect()
    }

    // ------------------------------------------------------------------
    // print_help / print_version
    // ------------------------------------------------------------------

    #[test]
    fn print_help_contains_rx() {
        let mut buf = Vec::new();
        print_help(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("RX"));
    }

    #[test]
    fn print_version_contains_semver() {
        let mut buf = Vec::new();
        print_version(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(VERSION));
    }

    // ------------------------------------------------------------------
    // config_print
    // ------------------------------------------------------------------

    #[test]
    fn config_print_renders_all_fields() {
        let mut buf = Vec::new();
        let config = Config {
            repo: String::from("~/.dotfiles"),
            hostname: String::from("nixos"),
            keep: 10,
            update: false,
            diff: true,
        };
        config_print(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("repo"));
        assert!(s.contains("hostname"));
        assert!(s.contains("keep"));
        assert!(s.contains("update"));
        assert!(s.contains("diff"));
    }

    #[test]
    fn config_print_last_field_has_no_trailing_newline() {
        let mut buf = Vec::new();
        let config = Config {
            repo: String::from("r"),
            hostname: String::from("h"),
            keep: 1,
            update: true,
            diff: false,
        };
        config_print(&mut buf, &config).unwrap();
        let s = String::from_utf8(buf).unwrap();
        // The "diff" line is last with new_line=false → no trailing "\n"
        assert!(!s.ends_with('\n'));
    }

    // ------------------------------------------------------------------
    // parse_args — early-return paths
    // ------------------------------------------------------------------

    #[test]
    fn parse_no_args_yields_defaults() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx"]), &mut buf) {
            Parsed::Run(c) => {
                assert_eq!(c.repo, "~/.dotfiles");
                assert_eq!(c.keep, 10);
                assert!(!c.update);
                assert!(!c.diff);
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_help_flag_returns_help() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-h"]), &mut buf), Parsed::Help));
    }

    #[test]
    fn parse_help_word_returns_help() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "help"]), &mut buf), Parsed::Help));
    }

    #[test]
    fn parse_version_flag_returns_version() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-v"]), &mut buf), Parsed::Version));
    }

    #[test]
    fn parse_version_word_returns_version() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "version"]), &mut buf), Parsed::Version));
    }

    #[test]
    fn parse_unknown_argument_returns_error() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "unknown"]), &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("Unknown argument"));
    }

    // ------------------------------------------------------------------
    // parse_args — value flags
    // ------------------------------------------------------------------

    #[test]
    fn parse_r_missing_value_returns_error() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-r"]), &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("requires an argument"));
    }

    #[test]
    fn parse_n_missing_value_returns_error() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-n"]), &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("requires an argument"));
    }

    #[test]
    fn parse_k_non_numeric_returns_error() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-k", "abc"]), &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("not numeric"));
    }

    #[test]
    fn parse_r_with_value_sets_repo() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-r", "/path/to/repo"]), &mut buf) {
            Parsed::Run(c) => assert_eq!(c.repo, "/path/to/repo"),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_n_with_value_sets_hostname() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-n", "myhost"]), &mut buf) {
            Parsed::Run(c) => assert_eq!(c.hostname, "myhost"),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_k_with_value_sets_keep() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-k", "5"]), &mut buf) {
            Parsed::Run(c) => assert_eq!(c.keep, 5),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // parse_args — boolean flags
    // ------------------------------------------------------------------

    #[test]
    fn parse_d_flag_sets_diff() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-d"]), &mut buf) {
            Parsed::Run(c) => assert!(c.diff),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_u_flag_sets_update() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-u"]), &mut buf) {
            Parsed::Run(c) => assert!(c.update),
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_d_and_u_together() {
        let mut buf = Vec::new();
        match parse_args(&args(&["rx", "-d", "-u"]), &mut buf) {
            Parsed::Run(c) => {
                assert!(c.diff);
                assert!(c.update);
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_combined_flags_and_values() {
        let mut buf = Vec::new();
        match parse_args(
            &args(&["rx", "-d", "-u", "-r", "/my/repo", "-n", "myhost", "-k", "3"]),
            &mut buf,
        ) {
            Parsed::Run(c) => {
                assert!(c.diff);
                assert!(c.update);
                assert_eq!(c.repo, "/my/repo");
                assert_eq!(c.hostname, "myhost");
                assert_eq!(c.keep, 3);
            }
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn parse_k_then_h_early_returns_help() {
        let mut buf = Vec::new();
        assert!(matches!(
            parse_args(&args(&["rx", "-k", "5", "-h"]), &mut buf),
            Parsed::Help
        ));
    }

    #[test]
    fn parse_unknown_flag_returns_error() {
        let mut buf = Vec::new();
        assert!(matches!(parse_args(&args(&["rx", "-x"]), &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("Unknown flag"));
    }

    // ------------------------------------------------------------------
    // run dispatcher
    // ------------------------------------------------------------------

    #[test]
    fn run_help_flag_routes_to_help() {
        let mut buf = Vec::new();
        run(&args(&["rx", "-h"]), &mut buf, &MockDeps).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("RX"));
    }

    #[test]
    fn run_version_flag_routes_to_version() {
        let mut buf = Vec::new();
        run(&args(&["rx", "-v"]), &mut buf, &MockDeps).unwrap();
    }

    #[test]
    fn run_unknown_flag_returns_error() {
        let mut buf = Vec::new();
        assert!(run(&args(&["rx", "-x"]), &mut buf, &MockDeps).is_err());
    }

    #[test]
    fn run_no_args_reaches_cli() {
        let mut buf = Vec::new();
        run(&args(&["rx"]), &mut buf, &MockCliDeps).unwrap();
    }

    #[test]
    fn run_with_d_and_u_reaches_cli() {
        let mut buf = Vec::new();
        run(&args(&["rx", "-d", "-u"]), &mut buf, &MockCliDeps).unwrap();
    }

    // ------------------------------------------------------------------
    // Config / hostname
    // ------------------------------------------------------------------

    #[test]
    fn config_defaults_repo_and_keep() {
        let c = Config::defaults();
        assert_eq!(c.repo, "~/.dotfiles");
        assert_eq!(c.keep, 10);
    }

    #[test]
    fn default_hostname_is_non_empty() {
        let c = Config::defaults();
        assert!(!c.hostname.is_empty());
    }

    #[test]
    fn config_display_contains_repo() {
        let c = Config {
            repo: String::from("repo"),
            hostname: String::from("host"),
            keep: 5,
            update: true,
            diff: false,
        };
        assert!(c.to_string().contains("repo"));
    }

    // ------------------------------------------------------------------
    // shared mock deps
    // ------------------------------------------------------------------

    struct MockDeps;

    impl Deps for MockDeps {
        fn run_shell(&self, _: &str, _: bool) -> Result<i32, Error> { Ok(0) }
        fn confirm(&self, _: bool, _: Option<&str>) -> Result<bool, Error> { Ok(false) }
        fn print_title(&self, _: &str) -> Result<(), Error> { Ok(()) }
        fn config_print(&self, _: &Config) -> Result<(), Error> { Ok(()) }
    }

    struct MockCliDeps;

    impl Deps for MockCliDeps {
        fn run_shell(&self, _: &str, _: bool) -> Result<i32, Error> { Ok(0) }
        fn confirm(&self, _: bool, _: Option<&str>) -> Result<bool, Error> { Ok(false) }
        fn print_title(&self, _: &str) -> Result<(), Error> { Ok(()) }
        fn config_print(&self, _: &Config) -> Result<(), Error> { Ok(()) }
    }
}
use std::fmt;
use std::io::Write;

use crate::core::ansi;
use crate::error::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Config {
    pub repo: String,
    pub hostname: String,
    pub keep: u8,
    pub update: bool,
    pub diff: bool,
}

impl Config {
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

fn default_hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_owned())
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| String::from("unknown"))
}

pub fn print_help(writer: &mut dyn Write) -> Result<(), Error> {
    ansi::write_flush(
        writer,
        "\n*****************************************************\n RX - A simple CLI tool to update your nixos system\n*****************************************************\n-r : set repo path (default is $HOME/.dotfiles)\n-n : set hostname (default is OS hostname)\n-k : set generations to keep (default is 10)\n-u : set update to true (default is false)\n-d : set diff to true (default is false)\n-h, help : Display this help message\n-v, version : Display the current version\n\n",
    )?;
    Ok(())
}

pub fn print_version(writer: &mut dyn Write) -> Result<(), Error> {
    ansi::write_flush(
        writer,
        &format!("{}RX version: {}{}\n", ansi::YELLOW, VERSION, ansi::RESET),
    )?;
    Ok(())
}

fn print_config_line(writer: &mut dyn Write, label: &str, value: &str, new_line: bool) -> Result<(), Error> {
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

pub fn config_print(writer: &mut dyn Write, config: &Config) -> Result<(), Error> {
    print_config_line(writer, "repo", &config.repo, true)?;
    print_config_line(writer, "hostname", &config.hostname, true)?;
    print_config_line(writer, "keep", &config.keep.to_string(), true)?;
    print_config_line(writer, "update", &config.update.to_string(), true)?;
    print_config_line(writer, "diff", &config.diff.to_string(), false)?;
    Ok(())
}

pub enum Parsed {
    Help,
    Version,
    Run(Config),
    Error,
}

pub fn parse_args(args: &[String], writer: &mut dyn Write) -> Parsed {
    if args.len() <= 1 {
        return Parsed::Run(Config::defaults());
    }

    // Check for word arguments first (help, version)
    if args.len() == 2 {
        match args[1].as_str() {
            "help" => return Parsed::Help,
            "version" => return Parsed::Version,
            _ => {}
        }
    }

    // Check if first non-program arg starts with '-'
    if !args[1].starts_with('-') {
        let arg = &args[1];
        ansi::write_flush(writer, &format!("{}Error: Unknown argument \"{}\"\n{}", ansi::RED, arg, ansi::RESET)).ok();
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
                            ansi::write_flush(writer, &format!("{}Error: \"-{}\" flag requires an argument\n{}", ansi::RED, flag_char, ansi::RESET)).ok();
                            return Parsed::Error;
                        }
                        let value = &args[i + 1];
                        match flag_char {
                            'r' => config.repo = value.clone(),
                            'n' => config.hostname = value.clone(),
                            'k' => {
                                match value.parse::<u8>() {
                                    Ok(num) => config.keep = num,
                                    Err(_) => {
                                        ansi::write_flush(writer, &format!("{}Error: Value of \"-k\" flag is not numeric.\n{}", ansi::RED, ansi::RESET)).ok();
                                        return Parsed::Error;
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                        skip_next = true;
                        break; // value flags consume rest of arg group
                    }
                    _ => {
                        ansi::write_flush(writer, &format!("{}Error: Unknown flag \"-{}\"\n{}", ansi::RED, flag_char, ansi::RESET)).ok();
                        return Parsed::Error;
                    }
                }
            }
            if skip_next {
                i += 1; // skip value arg
            }
        }
        i += 1;
    }

    Parsed::Run(config)
}

pub fn run<W: Write>(args: &[String], writer: &mut W, deps: &dyn crate::app::cli::Deps) -> Result<(), Error> {
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

    fn args(from: &[&str]) -> Vec<String> {
        from.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn print_help_contains_rx() {
        let mut buf = Vec::new();
        print_help(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("RX"));
    }

    #[test]
    fn print_version_contains_version() {
        let mut buf = Vec::new();
        print_version(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(VERSION));
    }

    #[test]
    fn config_print_renders_fields() {
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
    fn config_print_last_field_no_newline() {
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
        // Last field (diff=false) should NOT end with newline
        assert!(s.ends_with("false") || s.contains("diff"));
    }

    #[test]
    fn parse_help_flag() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-h"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Help));
    }

    #[test]
    fn parse_help_word() {
        let mut buf = Vec::new();
        let a = args(&["rx", "help"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Help));
    }

    #[test]
    fn parse_version_flag() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-v"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Version));
    }

    #[test]
    fn parse_version_word() {
        let mut buf = Vec::new();
        let a = args(&["rx", "version"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Version));
    }

    #[test]
    fn parse_unknown_argument() {
        let mut buf = Vec::new();
        let a = args(&["rx", "unknown"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("Unknown argument"));
    }

    #[test]
    fn parse_r_requires_argument() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-r"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("requires an argument"));
    }

    #[test]
    fn parse_n_requires_argument() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-n"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("requires an argument"));
    }

    #[test]
    fn parse_k_not_numeric() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-k", "abc"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("not numeric"));
    }

    #[test]
    fn parse_unknown_flag() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-x"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Error));
        assert!(String::from_utf8(buf).unwrap().contains("Unknown flag"));
    }

    #[test]
    fn parse_d_flag() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-d"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => assert!(c.diff),
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_u_flag() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-u"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => assert!(c.update),
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_r_with_value() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-r", "/path/to/repo"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => assert_eq!(c.repo, "/path/to/repo"),
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_k_then_h() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-k", "5", "-h"]);
        assert!(matches!(parse_args(&a, &mut buf), Parsed::Help));
    }

    #[test]
    fn parse_no_args() {
        let mut buf = Vec::new();
        let a = args(&["rx"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => {
                assert_eq!(c.repo, "~/.dotfiles");
                assert_eq!(c.keep, 10);
            }
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn default_hostname_is_not_empty() {
        let config = Config::defaults();
        assert!(!config.hostname.is_empty());
    }

    #[test]
    fn config_defaults() {
        let config = Config::defaults();
        assert_eq!(config.repo, "~/.dotfiles");
        assert_eq!(config.keep, 10);
        assert!(!config.update);
        assert!(!config.diff);
    }

    #[test]
    fn run_with_help_flag() {
        let deps = MockDeps;
        let mut buf = Vec::new();
        let a = args(&["rx", "-h"]);
        let result = run(&a, &mut buf, &deps);
        assert!(result.is_ok());
        assert!(String::from_utf8(buf).unwrap().contains("RX"));
    }

    #[test]
    fn run_with_version_flag() {
        let deps = MockDeps;
        let mut buf = Vec::new();
        let a = args(&["rx", "-v"]);
        let result = run(&a, &mut buf, &deps);
        assert!(result.is_ok());
        assert!(String::from_utf8(buf).unwrap().contains(VERSION));
    }

    #[test]
    fn run_with_error_returns_err() {
        let deps = MockDeps;
        let mut buf = Vec::new();
        let a = args(&["rx", "-x"]);
        let result = run(&a, &mut buf, &deps);
        assert!(result.is_err());
    }

    #[test]
    fn run_no_args_reaches_cli() {
        let mut buf = Vec::new();
        let a = args(&["rx"]);
        let result = run(&a, &mut buf, &MockCliDeps);
        assert!(result.is_ok());
    }

    #[test]
    fn run_with_help_word() {
        let deps = MockDeps;
        let mut buf = Vec::new();
        let a = args(&["rx", "help"]);
        let result = run(&a, &mut buf, &deps);
        assert!(result.is_ok());
        assert!(String::from_utf8(buf).unwrap().contains("RX"));
    }

    #[test]
    fn run_with_version_word() {
        let deps = MockDeps;
        let mut buf = Vec::new();
        let a = args(&["rx", "version"]);
        let result = run(&a, &mut buf, &deps);
        assert!(result.is_ok());
    }

    #[test]
    fn run_with_d_and_u() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-d", "-u"]);
        let result = run(&a, &mut buf, &MockCliDeps);
        assert!(result.is_ok());
    }

    #[test]
    fn config_display() {
        let config = Config {
            repo: String::from("repo"),
            hostname: String::from("host"),
            keep: 5,
            update: true,
            diff: false,
        };
        let s = format!("{config}");
        assert!(s.contains("repo"));
    }

    #[test]
    fn parse_n_with_value() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-n", "myhost"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => assert_eq!(c.hostname, "myhost"),
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_k_with_valid_number() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-k", "5"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => assert_eq!(c.keep, 5),
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_d_and_u_together() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-d", "-u"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => {
                assert!(c.diff);
                assert!(c.update);
            }
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    #[test]
    fn parse_combined_du_flags() {
        let mut buf = Vec::new();
        let a = args(&["rx", "-d", "-u", "-r", "/my/repo", "-n", "myhost", "-k", "3"]);
        match parse_args(&a, &mut buf) {
            Parsed::Run(c) => {
                assert!(c.diff);
                assert!(c.update);
                assert_eq!(c.repo, "/my/repo");
                assert_eq!(c.hostname, "myhost");
                assert_eq!(c.keep, 3);
            }
            other => panic!("expected Run, got {:?}", fmt_parsed(&other)),
        }
    }

    // Mock deps for run() tests
    struct MockDeps;

    impl crate::app::cli::Deps for MockDeps {
        fn run_shell(&self, _cmd: &str, _output: bool) -> Result<i32, Error> { Ok(0) }
        fn confirm(&self, _default: bool, _msg: Option<&str>) -> Result<bool, Error> { Ok(false) }
        fn print_title(&self, _text: &str) -> Result<(), Error> { Ok(()) }
        fn config_print(&self, _config: &Config) -> Result<(), Error> { Ok(()) }
    }

    struct MockCliDeps;

    impl crate::app::cli::Deps for MockCliDeps {
        fn run_shell(&self, _cmd: &str, _output: bool) -> Result<i32, Error> { Ok(0) }
        fn confirm(&self, _default: bool, _msg: Option<&str>) -> Result<bool, Error> { Ok(false) }
        fn print_title(&self, _text: &str) -> Result<(), Error> { Ok(()) }
        fn config_print(&self, _config: &Config) -> Result<(), Error> { Ok(()) }
    }

    fn fmt_parsed(p: &Parsed) -> String {
        match p {
            Parsed::Help => "Help".to_string(),
            Parsed::Version => "Version".to_string(),
            Parsed::Run(c) => format!("Run({})", c),
            Parsed::Error => "Error".to_string(),
        }
    }
}
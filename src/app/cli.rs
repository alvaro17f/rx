use std::io::Write;

use crate::core::commands;
use crate::core::ansi;
use crate::error::Error;

use crate::app::init::Config;

pub trait Deps {
    fn run_shell(&self, cmd: &str, output: bool) -> Result<i32, Error>;
    fn confirm(&self, default: bool, msg: Option<&str>) -> Result<bool, Error>;
    fn print_title(&self, text: &str) -> Result<(), Error>;
    fn config_print(&self, config: &Config) -> Result<(), Error>;
}

pub struct RealDeps;

impl Deps for RealDeps {
    fn run_shell(&self, cmd: &str, output: bool) -> Result<i32, Error> {
        crate::core::process::run(cmd, output)
    }

    fn confirm(&self, default: bool, msg: Option<&str>) -> Result<bool, Error> {
        let stdin = std::io::stdin();
        let mut reader = stdin.lock();
        let mut stdout = std::io::stdout();
        crate::core::ui::confirm(&mut reader, &mut stdout, default, msg)
    }

    fn print_title(&self, text: &str) -> Result<(), Error> {
        let mut stdout = std::io::stdout();
        crate::core::ui::print_title(&mut stdout, text)
    }

    fn config_print(&self, config: &Config) -> Result<(), Error> {
        let mut stdout = std::io::stdout();
        crate::app::init::config_print(&mut stdout, config)
    }
}

pub fn cli(writer: &mut dyn Write, config: &Config, deps: &dyn Deps) -> Result<(), Error> {
    deps.print_title("RX Configuration")?;
    deps.config_print(config)?;

    if !deps.confirm(true, None)? {
        return Ok(());
    }

    deps.print_title("Git Pull")?;
    let status = deps.run_shell(&commands::git_pull(&config.repo), true)?;
    if status != 0 {
        ansi::write_flush(writer, &format!("{}Failed to pull changes{}\n", ansi::RED, ansi::RESET))?;
        return Err(Error::GitPullFailed);
    }

    if config.update {
        deps.print_title("Nix Update")?;
        deps.run_shell(&commands::nix_update(&config.repo), true)?;
    }

    let diff_status = deps.run_shell(&commands::git_diff(&config.repo), false)?;
    if diff_status == 1 {
        deps.print_title("Git Changes")?;
        deps.run_shell(&commands::git_status(&config.repo), true)?;

        if deps.confirm(true, Some("Do you want to add these changes to the stage?"))? {
            match deps.run_shell(&commands::git_add(&config.repo), true) {
                Ok(_) => {
                    ansi::write_flush(writer, &format!("{}Changes added to git stage successfully{}\n", ansi::GREEN, ansi::RESET))?;
                }
                Err(e) => {
                    ansi::write_flush(writer, &format!("Failed to add changes to the stage: {}\n", e))?;
                }
            }
        } else {
            ansi::write_flush(writer, &format!("{}Changes not added to stage{}\n", ansi::RED, ansi::RESET))?;
        }
    }

    deps.print_title("Nixos Rebuild")?;
    deps.run_shell(&commands::nix_rebuild(&config.repo, &config.hostname), true)?;

    deps.run_shell(&commands::nix_keep(config.keep), true)?;

    if config.diff {
        deps.print_title("Nix Diff")?;
        deps.run_shell(commands::nix_diff(), true)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDeps {
        run_result: Box<dyn Fn(&str, bool) -> Result<i32, Error>>,
        confirm_result: Box<dyn Fn(bool, Option<&str>) -> Result<bool, Error>>,
        print_title_fn: Box<dyn Fn(&str) -> Result<(), Error>>,
        config_print_fn: Box<dyn Fn(&Config) -> Result<(), Error>>,
    }

    impl Deps for MockDeps {
        fn run_shell(&self, cmd: &str, output: bool) -> Result<i32, Error> {
            (self.run_result)(cmd, output)
        }
        fn confirm(&self, default: bool, msg: Option<&str>) -> Result<bool, Error> {
            (self.confirm_result)(default, msg)
        }
        fn print_title(&self, text: &str) -> Result<(), Error> {
            (self.print_title_fn)(text)
        }
        fn config_print(&self, config: &Config) -> Result<(), Error> {
            (self.config_print_fn)(config)
        }
    }

    fn default_config() -> Config {
        Config {
            repo: String::from("r"),
            hostname: String::from("h"),
            keep: 1,
            update: false,
            diff: false,
        }
    }

    fn mock_deps_ok() -> MockDeps {
        MockDeps {
            run_result: Box::new(|cmd, _| {
                if cmd.contains("diff --exit-code") {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }),
            confirm_result: Box::new(|_, _| Ok(true)),
            print_title_fn: Box::new(|_| Ok(())),
            config_print_fn: Box::new(|_| Ok(())),
        }
    }

    #[test]
    fn confirm_false_early_return() {
        let deps = MockDeps {
            confirm_result: Box::new(|_, _| Ok(false)),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let config = default_config();
        cli(&mut buf, &config, &deps).unwrap();
    }

    #[test]
    fn confirm_decline_add_changes() {
        let deps = MockDeps {
            run_result: Box::new(|cmd, _| {
                if cmd.contains("diff --exit-code") {
                    return Ok(1);
                }
                Ok(0)
            }),
            confirm_result: Box::new(|default, msg| {
                if msg.is_some() {
                    Ok(false)
                } else {
                    Ok(default)
                }
            }),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let config = default_config();
        cli(&mut buf, &config, &deps).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Changes not added to stage"));
    }

    #[test]
    fn git_pull_fails() {
        let deps = MockDeps {
            run_result: Box::new(|_, _| Ok(1)),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let config = default_config();
        let result = cli(&mut buf, &config, &deps);
        assert!(result.is_err());
    }

    #[test]
    fn update_and_diff_flags() {
        let deps = MockDeps {
            run_result: Box::new(|cmd, _| {
                if cmd.contains("diff --exit-code") {
                    return Ok(1);
                }
                Ok(0)
            }),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let mut config = default_config();
        config.update = true;
        config.diff = true;
        cli(&mut buf, &config, &deps).unwrap();
    }

    #[test]
    fn no_git_changes() {
        let deps = MockDeps {
            run_result: Box::new(|_, _| Ok(0)),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let config = default_config();
        cli(&mut buf, &config, &deps).unwrap();
    }

    #[test]
    fn git_add_failure_caught() {
        let deps = MockDeps {
            run_result: Box::new(|cmd, _| {
                if cmd.contains("add .") {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "mock error",
                    )));
                }
                if cmd.contains("diff --exit-code") {
                    return Ok(1);
                }
                Ok(0)
            }),
            ..mock_deps_ok()
        };
        let mut buf = Vec::new();
        let config = default_config();
        cli(&mut buf, &config, &deps).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Failed to add changes to the stage"));
    }

    #[test]
    fn real_deps_run_shell_true() {
        let deps = RealDeps;
        assert_eq!(deps.run_shell("true", false).unwrap(), 0);
    }

    #[test]
    fn real_deps_print_title() {
        let deps = RealDeps;
        assert!(deps.print_title("Test").is_ok());
    }

    #[test]
    fn real_deps_config_print() {
        let deps = RealDeps;
        let config = Config {
            repo: String::from("r"),
            hostname: String::from("h"),
            keep: 1,
            update: false,
            diff: false,
        };
        assert!(deps.config_print(&config).is_ok());
    }
}
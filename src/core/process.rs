use std::process::{Command, Stdio};

use crate::error::Error;

/// Run a shell command and return its exit code.
///
/// When `output` is `false`, stdout and stderr are redirected to `/dev/null`.
/// A process killed by a signal returns `1`.
pub fn run(cmd: &str, output: bool) -> Result<i32, Error> {
    let stdout = if output { Stdio::inherit() } else { Stdio::null() };
    let stderr = if output { Stdio::inherit() } else { Stdio::null() };
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::inherit())
        .stdout(stdout)
        .stderr(stderr)
        .status()
        .map_err(Error::Io)?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_true_returns_zero() {
        assert_eq!(run("true", true).unwrap(), 0);
    }

    #[test]
    fn run_false_returns_one() {
        assert_eq!(run("false", true).unwrap(), 1);
    }

    #[test]
    fn run_echo_with_output_false_returns_zero() {
        assert_eq!(run("echo hello", false).unwrap(), 0);
    }

    #[test]
    fn run_empty_command_returns_zero() {
        assert_eq!(run("", false).unwrap(), 0);
    }
}
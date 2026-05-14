use std::process::{Command, Stdio};

use crate::error::Error;

pub fn run(cmd: &str, output: bool) -> Result<i32, Error> {
    let stdout = if output {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
    let stderr = if output {
        Stdio::inherit()
    } else {
        Stdio::null()
    };
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
    fn run_true() {
        assert_eq!(run("true", true).unwrap(), 0);
    }

    #[test]
    fn run_false() {
        assert_eq!(run("false", true).unwrap(), 1);
    }

    #[test]
    fn run_echo_suppressed() {
        assert_eq!(run("echo hello", false).unwrap(), 0);
    }

    #[test]
    fn run_empty_command() {
        assert_eq!(run("", false).unwrap(), 0);
    }
}
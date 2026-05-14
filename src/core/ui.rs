use std::io::{BufRead, Write};

use crate::core::ansi;
use crate::error::Error;

pub fn print_title(writer: &mut dyn Write, text: &str) -> Result<(), Error> {
    let border: String = "*".repeat(text.len() + 4);
    ansi::write_flush(
        writer,
        &format!(
            "{}\n{}\n* {}{}{} *\n{}\n{}",
            ansi::BLUE, border, ansi::RED, text, ansi::BLUE, border, ansi::RESET
        ),
    )?;
    Ok(())
}

pub fn confirm(reader: &mut dyn BufRead, writer: &mut dyn Write, default: bool, msg: Option<&str>) -> Result<bool, Error> {
    write_confirm_prompt(writer, default, msg)?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    parse_confirm_response(line.trim(), default)
}

fn write_confirm_prompt(writer: &mut dyn Write, default: bool, msg: Option<&str>) -> Result<(), Error> {
    let hint = if default {
        format!("{}(Y/n){}", ansi::GREEN, ansi::RESET)
    } else {
        format!("{}(y/N){}", ansi::RED, ansi::RESET)
    };
    if let Some(value) = msg {
        ansi::write_flush(writer, &format!("\n{}{}{} {}: ", ansi::YELLOW, value, ansi::RESET, hint))?;
    } else {
        ansi::write_flush(writer, &format!("\n\n{}Proceed?{} {}: ", ansi::YELLOW, ansi::RESET, hint))?;
    }
    Ok(())
}

pub fn parse_confirm_response(line: &str, default: bool) -> Result<bool, Error> {
    let response = line.to_lowercase();
    if response == "y" || response == "yes" {
        return Ok(true);
    }
    if response == "n" || response == "no" {
        return Ok(false);
    }
    if response.is_empty() {
        return Ok(default);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_title_writes_output() {
        let mut buf = Vec::new();
        print_title(&mut buf, "RX").unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("RX"));
        assert!(s.contains(ansi::BLUE));
        assert!(s.contains(ansi::RED));
        assert!(s.contains(ansi::RESET));
    }

    #[test]
    fn parse_confirm_y() {
        assert_eq!(parse_confirm_response("y", false).unwrap(), true);
        assert_eq!(parse_confirm_response("Y", false).unwrap(), true);
    }

    #[test]
    fn parse_confirm_yes() {
        assert_eq!(parse_confirm_response("yes", false).unwrap(), true);
    }

    #[test]
    fn parse_confirm_n() {
        assert_eq!(parse_confirm_response("n", true).unwrap(), false);
    }

    #[test]
    fn parse_confirm_no() {
        assert_eq!(parse_confirm_response("no", true).unwrap(), false);
    }

    #[test]
    fn parse_confirm_empty_default_true() {
        assert_eq!(parse_confirm_response("", true).unwrap(), true);
    }

    #[test]
    fn parse_confirm_empty_default_false() {
        assert_eq!(parse_confirm_response("", false).unwrap(), false);
    }

    #[test]
    fn parse_confirm_garbage() {
        assert_eq!(parse_confirm_response("maybe", true).unwrap(), false);
        assert_eq!(parse_confirm_response("maybe", false).unwrap(), false);
    }

    #[test]
    fn confirm_with_input_y() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(confirm(&mut reader, &mut writer, false, None).unwrap());
    }

    #[test]
    fn confirm_with_input_n() {
        let input = b"n\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(!confirm(&mut reader, &mut writer, true, None).unwrap());
    }

    #[test]
    fn confirm_with_message() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        let result = confirm(&mut reader, &mut writer, false, Some("Sure")).unwrap();
        assert!(result);
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Sure"));
    }

    #[test]
    fn confirm_default_true_empty() {
        let input = b"\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(confirm(&mut reader, &mut writer, true, None).unwrap());
    }
}
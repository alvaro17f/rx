use std::io::{BufRead, Write};

use crate::core::ansi;
use crate::error::Error;

/// Print a banner title with ANSI-colored borders.
pub fn print_title<W: Write>(writer: &mut W, text: &str) -> Result<(), Error> {
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

/// Prompt user for confirmation on `reader`/`writer`.
///
/// `default` is returned when the user sends an empty line.
pub fn confirm<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    default: bool,
    msg: Option<&str>,
) -> Result<bool, Error> {
    write_confirm_prompt(writer, default, msg)?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    parse_confirm_response(line.trim(), default)
}

fn write_confirm_prompt<W: Write>(
    writer: &mut W,
    default: bool,
    msg: Option<&str>,
) -> Result<(), Error> {
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

/// Parse a confirmation response line.
///
/// Returns `default` for empty input, `true` for "y"/"yes", `false` for
/// "n"/"no" or any other input.
pub fn parse_confirm_response(line: &str, default: bool) -> Result<bool, Error> {
    match line.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        "" => Ok(default),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn print_title_contains_text_and_ansi_codes() {
        let mut buf = Vec::new();
        print_title(&mut buf, "RX").unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("RX"));
        assert!(s.contains(ansi::BLUE));
        assert!(s.contains(ansi::RED));
        assert!(s.contains(ansi::RESET));
    }

    #[test]
    fn parse_confirm_y_returns_true() {
        assert_eq!(parse_confirm_response("y", false).unwrap(), true);
    }

    #[test]
    fn parse_confirm_uppercase_y_returns_true() {
        assert_eq!(parse_confirm_response("Y", false).unwrap(), true);
    }

    #[test]
    fn parse_confirm_yes_returns_true() {
        assert_eq!(parse_confirm_response("yes", false).unwrap(), true);
    }

    #[test]
    fn parse_confirm_n_returns_false() {
        assert_eq!(parse_confirm_response("n", true).unwrap(), false);
    }

    #[test]
    fn parse_confirm_no_returns_false() {
        assert_eq!(parse_confirm_response("no", true).unwrap(), false);
    }

    #[test]
    fn parse_confirm_empty_with_true_default_returns_true() {
        assert_eq!(parse_confirm_response("", true).unwrap(), true);
    }

    #[test]
    fn parse_confirm_empty_with_false_default_returns_false() {
        assert_eq!(parse_confirm_response("", false).unwrap(), false);
    }

    #[test]
    fn parse_confirm_garbage_returns_false() {
        assert_eq!(parse_confirm_response("maybe", true).unwrap(), false);
        assert_eq!(parse_confirm_response("maybe", false).unwrap(), false);
    }

    #[test]
    fn confirm_reads_y_and_returns_true() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(confirm(&mut reader, &mut writer, false, None).unwrap());
    }

    #[test]
    fn confirm_reads_n_and_returns_false() {
        let input = b"n\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(!confirm(&mut reader, &mut writer, true, None).unwrap());
    }

    #[test]
    fn confirm_with_message_writes_message_to_writer() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        let result = confirm(&mut reader, &mut writer, false, Some("Sure")).unwrap();
        assert!(result);
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Sure"));
    }

    #[test]
    fn confirm_empty_line_uses_default_true() {
        let input = b"\n";
        let mut reader = &input[..];
        let mut writer = Vec::new();
        assert!(confirm(&mut reader, &mut writer, true, None).unwrap());
    }

    // ------------------------------------------------------------------
    // error propagation
    // ------------------------------------------------------------------

    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
        }
    }
    impl BufRead for FailingReader {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
        }
        fn consume(&mut self, _: usize) {}
    }

    #[test]
    fn print_title_error_propagation() {
        let mut writer = FailingWriter;
        assert!(print_title(&mut writer, "X").is_err());
    }

    #[test]
    fn confirm_error_from_writer() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = FailingWriter;
        assert!(confirm(&mut reader, &mut writer, false, None).is_err());
    }

    #[test]
    fn confirm_error_from_writer_with_message() {
        let input = b"y\n";
        let mut reader = &input[..];
        let mut writer = FailingWriter;
        assert!(confirm(&mut reader, &mut writer, false, Some("X")).is_err());
    }

    #[test]
    fn confirm_error_from_reader_read_line() {
        let mut reader = FailingReader;
        let mut writer = Vec::new();
        assert!(confirm(&mut reader, &mut writer, false, None).is_err());
    }
}
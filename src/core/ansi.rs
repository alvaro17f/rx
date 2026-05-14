use std::io::Write;

// ANSI escape codes. Unused constants are kept for parity with the original
// Zig source and potential future styling needs.

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
#[expect(dead_code, reason = "unused in current palette, kept for parity with Zig original")]
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
#[expect(dead_code, reason = "unused in current palette, kept for parity with Zig original")]
pub const GRAY: &str = "\x1b[37m";
#[expect(dead_code, reason = "unused in current palette, kept for parity with Zig original")]
pub const BLACK: &str = "\x1b[30m";
pub const RESET: &str = "\x1b[0m";
#[expect(dead_code, reason = "unused in current palette, kept for parity with Zig original")]
pub const BOLD: &str = "\x1b[1m";
#[expect(dead_code, reason = "unused in current palette, kept for parity with Zig original")]
pub const UNDERLINE: &str = "\x1b[4m";

/// Write `msg` to `writer` and flush immediately.
pub fn write_flush<W: Write + ?Sized>(writer: &mut W, msg: &str) -> Result<(), std::io::Error> {
    writer.write_all(msg.as_bytes())?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_constants_are_non_empty() {
        assert!(!RED.is_empty());
        assert!(!GREEN.is_empty());
        assert!(!YELLOW.is_empty());
        assert!(!BLUE.is_empty());
        assert!(!MAGENTA.is_empty());
        assert!(!CYAN.is_empty());
        assert!(!GRAY.is_empty());
        assert!(!BLACK.is_empty());
        assert!(!RESET.is_empty());
        assert!(!BOLD.is_empty());
        assert!(!UNDERLINE.is_empty());
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
        }
    }

    #[test]
    fn failing_writer_flush_is_err() {
        let mut w = FailingWriter;
        assert!(std::io::Write::flush(&mut w).is_err());
    }

    #[test]
    fn write_flush_writes_exact_bytes() {
        let mut buf = Vec::new();
        write_flush(&mut buf, "hello").unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn write_flush_propagates_write_error() {
        let mut writer = FailingWriter;
        assert!(write_flush(&mut writer, "x").is_err());
    }

    #[test]
    fn write_flush_propagates_flush_error_on_empty_write() {
        // Write succeeds but flush fails
        struct FlushFailingWriter;
        impl Write for FlushFailingWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"))
            }
        }
        let mut writer = FlushFailingWriter;
        assert!(write_flush(&mut writer, "x").is_err());
    }
}
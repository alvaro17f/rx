use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn help_flag() {
    let output = Command::new("./target/debug/rx")
        .arg("-h")
        .output()
        .expect("failed to run rx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RX"));
}

#[test]
fn version_flag() {
    let output = Command::new("./target/debug/rx")
        .arg("-v")
        .output()
        .expect("failed to run rx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn unknown_flag_exits_with_error() {
    let output = Command::new("./target/debug/rx")
        .arg("-x")
        .output()
        .expect("failed to run rx");
    assert!(!output.status.success());
}

#[test]
fn confirm_with_piped_stdin() {
    let mut child = Command::new("./target/debug/rx")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn rx");
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(b"n\n");
        drop(stdin);
    }
    let _ = child.wait();
}

#[test]
fn main_no_args_with_decline() {
    let _output = Command::new("./target/debug/rx")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("HOSTNAME", "testhost")
        .output()
        .expect("failed to run rx");
}

#[test]
fn version_word() {
    let output = Command::new("./target/debug/rx")
        .arg("version")
        .output()
        .expect("failed to run rx");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RX version"));
}

#[test]
fn help_word() {
    let output = Command::new("./target/debug/rx")
        .arg("help")
        .output()
        .expect("failed to run rx");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RX"));
}

#[test]
fn invalid_k_flag() {
    let output = Command::new("./target/debug/rx")
        .args(["-k", "abc"])
        .output()
        .expect("failed to run rx");
    assert!(!output.status.success());
}
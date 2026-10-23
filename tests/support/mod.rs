#![allow(dead_code)]

use std::io::Write;
use std::process::{Command, Output, Stdio};

pub const MULTI: &str = "year,Boys,Girls\n2007,183,202\n2008,231,199\n2009,0.3,5\n";
pub const SINGLE: &str = "year,Boys\n2007,183\n2008,231\n2009,0\n";

pub fn run(stdin: &str, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_termgraph"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

// Escapes are made visible so snapshots stay readable.
pub fn chart(stdin: &str, args: &[&str]) -> String {
    let args = [&["--colorize", "always"], args].concat();
    let output = run(stdin, &args);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().replace('\x1b', "␛")
}

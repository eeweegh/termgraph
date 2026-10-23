mod support;

use std::io::Write;
use support::{MULTI, SINGLE, run};

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn empty_stdin_exits_silently() {
    for input in ["", "\n  \n", "# only a comment\n", "@,Boys\n"] {
        let output = run(input, &[]);
        assert_eq!(output.status.code(), Some(0), "{input:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty(), "{input:?}");
    }
}

#[test]
fn empty_input_prints_no_title() {
    let output = run("", &["--title", "T"]);
    assert!(output.status.success() && output.stdout.is_empty());
}

#[test]
fn reads_a_file() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(SINGLE.as_bytes()).unwrap();
    let output = run("", &[file.path().to_str().unwrap()]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("2008"));
}

#[test]
fn empty_file_exits_silently() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let output = run("", &[file.path().to_str().unwrap()]);
    assert!(output.status.success() && output.stdout.is_empty());
}

#[test]
fn missing_file_fails() {
    let output = run("", &["/nonexistent/data.csv"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).starts_with("termgraph: error: could not read /nonexistent/data.csv"));
}

#[test]
fn invalid_number_fails() {
    let output = run("a,1\nb,x\n", &[]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output), "termgraph: error: invalid data point [x] in line [b,x]\n");
}

#[test]
fn ragged_rows_fail() {
    let output = run("a,1,2\nb,3\n", &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("label [b] has 1 values, expected 2"));
}

#[test]
fn wrong_color_count_fails() {
    let output = run(MULTI, &["--color", "red"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("color (1) and category (2)"));
}

#[test]
fn unknown_color_lists_choices() {
    let output = run(SINGLE, &["--color", "pink"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("invalid color pink, choose from green, red, yellow"));
}

#[test]
fn invalid_format_is_a_usage_error() {
    let output = run(SINGLE, &["--format", "{:>8.1f}"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("has no conversion, expected e.g. %5.2f"));
}

#[test]
fn version_comes_from_cargo() {
    let output = run("", &["--version"]);
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        concat!("termgraph ", env!("CARGO_PKG_VERSION"), "\n")
    );
}

#[test]
fn color_table_has_32_rows() {
    let output = run("", &["--colors"]);
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).lines().count(), 32);
}

#[test]
fn closed_pipe_is_not_an_error() {
    let rows = "row,1\n".repeat(20_000);
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_termgraph"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let feeder = std::thread::spawn(move || stdin.write_all(rows.as_bytes()));
    drop(child.stdout.take());
    let output = child.wait_with_output().unwrap();
    let _ = feeder.join();
    assert!(output.status.success(), "{}", stderr(&output));
}

#[test]
fn tg_script_matches_bin_tg() {
    let output = run("", &["--tg-script"]);
    let script = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/bin/tg")).unwrap();
    assert!(output.status.success() && output.stderr.is_empty());
    assert_eq!(String::from_utf8_lossy(&output.stdout), script);
    assert!(script.starts_with("#!/usr/bin/env zsh\n"));
}

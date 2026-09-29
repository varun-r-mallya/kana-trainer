//! End-to-end tests driving the real binary over stdin.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn data_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("kanatrain-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn run(dir: &PathBuf, args: &[&str], input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kanatrain"))
        .args(args)
        .env("XDG_DATA_HOME", dir)
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn help_prints_usage() {
    let out = run(&data_dir("help"), &["--help"], "");
    assert!(out.contains("--reset") && out.contains("--no-color"));
}

#[test]
fn wrong_answer_shows_comparison_and_report() {
    let dir = data_dir("wrong");
    // hiragana, single kana, 1 question, wrong answer, skip lock-in, no more rounds
    let out = run(&dir, &["--no-color"], "1\n1\n1\nxxx\n\nn\n");
    assert!(out.contains("YOU ANSWERED") && out.contains("CORRECT"));
    assert!(out.contains("Session report"));
    assert!(out.contains("Score: 0/1 (0%)"));
    assert!(out.contains("Your mistakes"));
    assert!(!out.contains('\u{2014}'));
    assert!(dir.join("kanatrain/stats.tsv").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn quitting_immediately_is_clean() {
    let dir = data_dir("quit");
    let out = run(&dir, &["--no-color"], "1\n1\n5\nq\n");
    assert!(out.contains("Score: 0/0"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn eof_does_not_hang_or_panic() {
    let dir = data_dir("eof");
    let out = run(&dir, &["--no-color"], "");
    assert!(out.contains("KANA TRAINER"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn weak_mode_needs_history_then_uses_it() {
    let dir = data_dir("weak");
    // No history yet: "w" is rejected, then quit.
    let out = run(&dir, &["--no-color"], "3\nw\nq\n");
    assert!(out.contains("No recorded mistakes yet"));
    // Make a mistake, then drill weak spots only.
    run(&dir, &["--no-color"], "1\n1\n1\nxxx\n\nn\n");
    let out = run(&dir, &["--no-color"], "1\nw\n1\nq\n");
    assert!(out.contains("weak spots only"));
    assert!(out.contains("1/1"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn reset_clears_stats() {
    let dir = data_dir("reset");
    run(&dir, &["--no-color"], "1\n1\n1\nxxx\n\nn\n");
    let file = dir.join("kanatrain/stats.tsv");
    assert!(file.exists());
    let out = run(&dir, &["--reset"], "");
    assert!(out.contains("Statistics cleared"));
    assert!(!file.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

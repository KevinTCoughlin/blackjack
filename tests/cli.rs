use std::io::Write;
use std::process::{Command, Stdio};

fn bj() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bj"))
}

#[test]
fn seeded_random_demo_is_reproducible() {
    let args = ["demo", "--seed", "42", "--strategy", "random", "-n", "50"];
    let first = bj().args(args).output().unwrap();
    let second = bj().args(args).output().unwrap();

    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn zero_demo_count_is_rejected() {
    let output = bj().args(["demo", "-n", "0"]).output().unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("0 is not in 1.."));
}

#[test]
fn malformed_state_returns_an_error_instead_of_panicking() {
    let new_game = bj().args(["new", "--seed", "123"]).output().unwrap();
    assert!(new_game.status.success());

    let mut state: serde_json::Value = serde_json::from_slice(&new_game.stdout).unwrap();
    state["phase"] = serde_json::json!({
        "type": "PlayerTurn",
        "data": { "hand_index": 999 }
    });

    let mut child = bj()
        .arg("hit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(&state).unwrap().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success());
    assert!(stderr.contains("Invalid game state"));
    assert!(!stderr.contains("panicked"));
}

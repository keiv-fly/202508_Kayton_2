use assert_cmd::Command;

#[test]
fn variable_persists_across_lines_and_is_printable() {
    let mut cmd = Command::cargo_bin("kayton_repl").expect("binary exists");
    // Provide two lines of input and then close stdin to terminate the REPL
    let assert = cmd.write_stdin("x = 1\nprint(x)\n").assert().success();

    let out = assert.get_output();
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&out.stdout));
    combined.push_str(&String::from_utf8_lossy(&out.stderr));

    assert!(
        combined.contains("1"),
        "Output did not contain expected value.\n{}",
        combined
    );
    assert!(
        !combined.contains("NameError"),
        "REPL reported NameError unexpectedly.\n{}",
        combined
    );
}

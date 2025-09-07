use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn uninstall_reports_unimplemented() {
    let mut cmd = Command::cargo_bin("kik").unwrap();
    cmd.args(["uninstall", "demo-crate"]);
    cmd.assert()
        .failure()
        .stderr(contains("Not yet implemented: uninstall demo-crate"));
}

#[test]
fn kernel_install_reports_unimplemented_with_custom_name() {
    let mut cmd = Command::cargo_bin("kik").unwrap();
    cmd.args(["kernel", "install", "--name", "custom"]);
    cmd.assert()
        .failure()
        .stderr(contains("Not yet implemented: kernel install (name='custom')"));
}

#[test]
fn kernel_install_reports_unimplemented_with_default_name() {
    let mut cmd = Command::cargo_bin("kik").unwrap();
    cmd.args(["kernel", "install"]);
    cmd.assert()
        .failure()
        .stderr(contains("Not yet implemented: kernel install (name='kayton')"));
}

#[test]
fn displays_help() {
    Command::cargo_bin("kik")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("Kayton environment manager"));
}

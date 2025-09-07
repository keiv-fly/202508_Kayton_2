use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;

#[test]
fn uninstall_reports_unimplemented() {
    let mut cmd = Command::cargo_bin("kik").unwrap();
    cmd.args(["uninstall", "demo-crate"]);
    cmd.assert()
        .failure()
        .stderr(contains("Not yet implemented: uninstall demo-crate"));
}

#[test]
fn kernel_install_creates_kernelspec_in_local_env() {
    let td = test_temp_dir("kik_kernel_install");
    // Create local env
    Command::cargo_bin("kik")
        .unwrap()
        .current_dir(&td)
        .args(["create", "local"])
        .assert()
        .success();

    // Install kernel with custom name; skip binary copy and jupyter registration
    let mut cmd = Command::cargo_bin("kik").unwrap();
    cmd.current_dir(&td)
        .env("KAYTON_ACTIVE_ENV", "local")
        .env("KIK_SKIP_KERNEL_COPY", "1")
        .env("KIK_SKIP_JUPYTER", "1")
        .args(["kernel", "install", "--name", "custom"]);
    cmd.assert()
        .success()
        .stdout(contains("Installed Jupyter kernel 'custom'"));

    let kernel_json = td
        .join(".kayton")
        .join("kernels")
        .join("custom")
        .join("kernel.json");
    assert!(
        kernel_json.exists(),
        "kernelspec was not created: {}",
        kernel_json.display()
    );
}

#[test]
fn kernel_uninstall_removes_kernelspec_in_local_env() {
    let td = test_temp_dir("kik_kernel_uninstall");
    // Create local env
    Command::cargo_bin("kik")
        .unwrap()
        .current_dir(&td)
        .args(["create", "local"])
        .assert()
        .success();
    // Install first
    Command::cargo_bin("kik")
        .unwrap()
        .current_dir(&td)
        .env("KAYTON_ACTIVE_ENV", "local")
        .env("KIK_SKIP_KERNEL_COPY", "1")
        .env("KIK_SKIP_JUPYTER", "1")
        .args(["kernel", "install", "--name", "custom"])
        .assert()
        .success();
    // Uninstall
    Command::cargo_bin("kik")
        .unwrap()
        .current_dir(&td)
        .env("KAYTON_ACTIVE_ENV", "local")
        .env("KIK_SKIP_JUPYTER", "1")
        .args(["kernel", "uninstall", "--name", "custom"])
        .assert()
        .success();

    let kernel_dir = td.join(".kayton").join("kernels").join("custom");
    assert!(
        !kernel_dir.exists(),
        "kernelspec dir still exists: {}",
        kernel_dir.display()
    );
}

fn test_temp_dir(prefix: &str) -> PathBuf {
    let base = std::env::temp_dir();
    // Use process id and monotonic time for uniqueness
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = base.join(format!("{}_{}_{}", prefix, pid, nanos));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
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

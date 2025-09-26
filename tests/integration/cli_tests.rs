use std::process::Command;
use tempfile::TempDir;
use assert_cmd::Command as AssertCmd;

fn setup_test_env() -> (TempDir, String) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite://{}", db_path.display());
    (temp_dir, db_url)
}

#[test]
fn test_cli_help_command() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("ERP CLI system"));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("erp-cli"));
}

#[test]
fn test_inventory_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["inventory", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Inventory management commands"));
}

#[test]
fn test_customers_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["customers", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Customer management commands"));
}

#[test]
fn test_sales_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["sales", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Sales management commands"));
}

#[test]
fn test_reports_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["reports", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Report generation commands"));
}

#[test]
fn test_config_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Configuration management commands"));
}

// Integration tests that require database setup would need more complex setup
// For now, we test the CLI parsing and help outputs

#[cfg(test)]
mod predicates {
    pub mod str {
        pub fn contains(expected: &str) -> impl predicates::Predicate<str> {
            predicates::str::contains(expected)
        }
    }
}
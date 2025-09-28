use assert_cmd::Command as AssertCmd;
use predicates::prelude::*;

#[test]
fn test_cli_help_command() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ERP CLI"));
}

#[test]
fn test_cli_version_command() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("erp"));
}

#[test]
fn test_inventory_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["inventory", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("인벤토리 관리"));
}

#[test]
fn test_customers_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["customers", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("고객 관리"));
}

#[test]
fn test_sales_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["sales", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("영업 관리"));
}

#[test]
fn test_reports_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["reports", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("보고서"));
}

#[test]
fn test_config_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("설정 관리"));
}

#[test]
fn test_migrate_help() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["migrate", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("마이그레이션"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.arg("invalid-command").assert().failure();
}

#[test]
fn test_global_options() {
    let mut cmd = AssertCmd::cargo_bin("erp").unwrap();
    cmd.args(["--log-level", "debug", "--help"])
        .assert()
        .success();
}

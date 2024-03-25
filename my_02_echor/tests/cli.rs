use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn dies_no_args() -> Result<()> {
    Command::cargo_bin("echor")?
        .assert()
        .failure()
        .stderr(predicates::str::contains("USAGE"));
    Ok(())
}

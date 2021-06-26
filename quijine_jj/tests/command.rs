use assert_cmd::Command;

#[test]
fn test_prop_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("jj")?;
    let assert = cmd.arg("_.x.y").write_stdin(r#"{"x":{"y":42}}"#).assert();
    assert.success().stdout("42\n").stderr("");
    Ok(())
}

#[test]
fn test_prop_error_success() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("jj")?;
    let assert = cmd.arg("_.x.y").write_stdin(r#"{}"#).assert();
    assert.success().stdout("");
    Ok(())
}

#[test]
fn test_compile_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("jj")?;
    let assert = cmd.arg("}").assert();
    assert.failure().code(3).stdout("");
    Ok(())
}

#[test]
fn test_parse_failure() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("jj")?;
    let assert = cmd.arg("_").write_stdin("}").assert();
    assert.failure().code(4).stdout("");
    Ok(())
}

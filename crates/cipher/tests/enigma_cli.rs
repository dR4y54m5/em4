use std::process::Command;

fn cipher() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cipher"))
}

#[test]
fn cli_reproduces_bdzgo() {
    let out = cipher()
        .args([
            "enigma",
            "--rotors",
            "I,II,III",
            "--reflector",
            "B",
            "--rings",
            "AAA",
            "--positions",
            "AAA",
            "--text",
            "AAAAA",
        ])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("BDZGO"));
}

#[test]
fn cli_rejects_bad_reflector() {
    let out = cipher()
        .args([
            "enigma",
            "--rotors",
            "I,II,III",
            "--reflector",
            "Z",
            "--rings",
            "AAA",
            "--positions",
            "AAA",
            "--text",
            "A",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(output.stdout)
        .unwrap()
        .trim()
        .get(0..8)
        .unwrap()
        .to_string();

    println!("cargo:rustc-env=COMMIT_HASH={}", git_hash);
}

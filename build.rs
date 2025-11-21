use std::env;
use std::process::Command;

struct Git;

impl Git {
    fn rev_parse() -> Result<String, &'static str> {
        let git_hash = {
            let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() else {
                return Err("could not run git command");
            };

            let Ok(git_hash) = String::from_utf8(output.stdout) else {
                return Err("could not parse utf8 string from output");
            };

            String::from(git_hash.trim())
        };

        let Some(partial_git_hash) = git_hash.get(0..8) else {
            return Err("could not get the first eight character of the git hash");
        };

        Ok(partial_git_hash.to_string())
    }
}

fn build_revision() -> String {
    if let Ok(revision) = env::var("BUILD_REVISION") {
        if !revision.is_empty() {
            return String::from(revision.trim());
        }
    }

    if let Ok(commit_hash) = Git::rev_parse() {
        return commit_hash;
    }

    return String::from("unkonwn");
}

fn main() {
    println!("cargo:rustc-env=COMMIT_HASH={}", build_revision());
}

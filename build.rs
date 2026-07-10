use regex::Regex;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
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
    if let Ok(revision) = env::var("BUILD_REVISION")
        && !revision.is_empty()
    {
        return String::from(revision.trim());
    }

    if let Ok(commit_hash) = Git::rev_parse() {
        return commit_hash;
    }

    String::from("unknown")
}

const SPACED_PATTERN: &str =
    r"(?<book>(\d )?[a-zA-Z]+) (?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";
const COMPACT_PATTERN: &str = r"(?<book>\d?[a-zA-Z]+)(?<chapter>\d+):(?<section>\d+)\s*(?<text>.+)";

/// Parses a verse text file and writes a `&[(&str, usize, usize, &str)]`
/// expression to `$OUT_DIR/{name}_verses.rs`. Panics (failing the build)
/// on any line that does not match the verse pattern.
fn generate_verses(out_dir: &Path, name: &str, source: &str, pattern: &str, header_lines: usize) {
    println!("cargo:rerun-if-changed={source}");

    let text = fs::read_to_string(source)
        .unwrap_or_else(|error| panic!("could not read {source}: {error}"));
    let re = Regex::new(pattern).unwrap();

    let mut generated = String::from("&[\n");
    for (index, line) in text.lines().enumerate().skip(header_lines) {
        let line_number = index + 1;
        let caps = re
            .captures(line)
            .unwrap_or_else(|| panic!("{source}:{line_number}: unparseable verse: {line:?}"));
        let chapter: usize = caps["chapter"].parse().unwrap_or_else(|error| {
            panic!("{source}:{line_number}: invalid chapter number: {error}")
        });
        let section: usize = caps["section"].parse().unwrap_or_else(|error| {
            panic!("{source}:{line_number}: invalid verse number: {error}")
        });
        writeln!(
            generated,
            "    ({:?}, {}, {}, {:?}),",
            caps["book"].trim(),
            chapter,
            section,
            caps["text"].trim(),
        )
        .unwrap();
    }
    generated.push_str("]\n");

    fs::write(out_dir.join(format!("{name}_verses.rs")), generated)
        .unwrap_or_else(|error| panic!("could not write generated verses for {name}: {error}"));
}

fn main() {
    println!("cargo:rustc-env=COMMIT_HASH={}", build_revision());
    println!("cargo:rerun-if-env-changed=BUILD_REVISION");
    println!("cargo:rerun-if-changed=.git/HEAD");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    if env::var("CARGO_FEATURE_KJV").is_ok() {
        generate_verses(out_dir, "kjv", "src/kjv/kjv.txt", COMPACT_PATTERN, 1);
    }
    if env::var("CARGO_FEATURE_ERV").is_ok() {
        generate_verses(out_dir, "erv", "src/erv/erv.txt", SPACED_PATTERN, 2);
    }
    if env::var("CARGO_FEATURE_WEB").is_ok() {
        generate_verses(out_dir, "web", "src/web/web.txt", SPACED_PATTERN, 2);
    }
}

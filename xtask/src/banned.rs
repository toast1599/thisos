//! The source check for the banned patterns in CONTRIBUTING.md that no
//! lint catches: `static mut` and `&'static mut`.

use std::fs;
use std::io;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::cmd::output;
use crate::root;

/// The one place `&'static mut` is allowed: the loader's entry signature,
/// which `bootloader_api::entry_point!` fixes. A pattern is exempt when it
/// is in this file and the line holding its `'static` names the loader's
/// `BootInfo`.
const LOADER_ENTRY: (&str, &str) = ("arch/x86_64/src/lib.rs", "bootloader_api::BootInfo");

/// Fails, naming each line, if any Rust source in the repository, tracked or
/// not yet added, has a banned pattern.
pub fn check() -> Result<()> {
    let files = output(Command::new("git").current_dir(root()).args([
        "ls-files",
        "--cached",
        "--others",
        "--exclude-standard",
        "*.rs",
    ]))?;
    let mut found = Vec::new();
    for path in files.lines() {
        if let Some(source) = source(path)? {
            found.extend(scan(path, &source));
        }
    }
    if !found.is_empty() {
        bail!(
            "banned patterns (CONTRIBUTING.md, Banned patterns):\n{}",
            found.join("\n")
        );
    }
    Ok(())
}

/// The text of `path`, relative to the repository root, or `None` when it
/// is gone: git still lists a file deleted but not yet staged.
fn source(path: &str) -> Result<Option<String>> {
    match fs::read_to_string(root().join(path)) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {path}")),
    }
}

/// The banned patterns in `source`, one `path:line: pattern` each, naming
/// the line of the pattern's first word. The words of a pattern may be on
/// different lines, which rustfmt would join, but not under rustfmt::skip.
fn scan(path: &str, source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let code = code(source);
    // code keeps every line break, so its line numbers are source's.
    let mut words = Vec::new();
    for (i, line) in code.lines().enumerate() {
        words.extend(
            line.split(|c: char| !(c.is_alphanumeric() || c == '_' || c == '\''))
                .filter(|word| !word.is_empty())
                .map(|word| (i, word)),
        );
    }
    let mut found = Vec::new();
    for pair in words.windows(2) {
        let (i, banned) = match pair {
            [(i, "static"), (_, "mut")] => (i, "static mut"),
            [(i, "'static"), (_, "mut")]
                if !(path == LOADER_ENTRY.0 && lines[*i].contains(LOADER_ENTRY.1)) =>
            {
                (i, "&'static mut")
            }
            _ => continue,
        };
        found.push(format!("{path}:{}: {banned}", i + 1));
    }
    found
}

/// `source` with its comments and the contents of its string and character
/// literals blanked to spaces, keeping line breaks, so that prose and test
/// inputs that name a pattern don't count.
fn code(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut code = String::with_capacity(source.len());
    let mut i = 0;
    while i < chars.len() {
        match skipped(&chars, i) {
            Some(end) => {
                code.extend(
                    chars[i..end]
                        .iter()
                        .map(|&c| if c == '\n' { c } else { ' ' }),
                );
                i = end;
            }
            None => {
                code.push(chars[i]);
                i += 1;
            }
        }
    }
    code
}

/// The end of the comment or literal that starts at `chars[i]`, or `None`
/// when none does. An unterminated one runs to the end of the source.
fn skipped(chars: &[char], i: usize) -> Option<usize> {
    let rest = &chars[i..];
    let after = |offset: usize| i + offset;
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    match rest {
        ['/', '/', ..] => Some(
            rest.iter()
                .position(|&c| c == '\n')
                .map_or(chars.len(), after),
        ),
        ['/', '*', ..] => {
            // Block comments nest.
            let mut depth = 0;
            let mut j = 0;
            while j < rest.len() {
                match rest[j..] {
                    ['/', '*', ..] => depth += 1,
                    ['*', '/', ..] => depth -= 1,
                    _ => {
                        j += 1;
                        continue;
                    }
                }
                j += 2;
                if depth == 0 {
                    return Some(after(j));
                }
            }
            Some(chars.len())
        }
        ['"', ..] => {
            let mut j = 1;
            while j < rest.len() {
                match rest[j] {
                    '\\' => j += 2,
                    '"' => return Some(after(j + 1)),
                    _ => j += 1,
                }
            }
            Some(chars.len())
        }
        ['r', ..] | ['b' | 'c', 'r', ..] if i == 0 || !ident(chars[i - 1]) => {
            // A raw string, r"...", r#"..."#, br#"..."# or cr#"..."#, has no
            // escapes and ends at a quote followed by as many #s as it
            // started with.
            let prefix = if rest[0] == 'r' { 1 } else { 2 };
            let hashes = rest[prefix..].iter().take_while(|&&c| c == '#').count();
            if rest.get(prefix + hashes) != Some(&'"') {
                return None;
            }
            let body = prefix + hashes + 1;
            let end = (body..rest.len()).find(|&j| {
                rest[j] == '"'
                    && rest[j + 1..]
                        .iter()
                        .take(hashes)
                        .filter(|&&c| c == '#')
                        .count()
                        == hashes
            });
            Some(end.map_or(chars.len(), |j| after(j + 1 + hashes)))
        }
        // A character literal; a quote not closed two characters on, with no
        // escape, starts a lifetime such as 'static, which stays code.
        ['\'', '\\', ..] => Some(
            rest.iter()
                .skip(3)
                .position(|&c| c == '\'')
                .map_or(chars.len(), |j| after(j + 4)),
        ),
        ['\'', _, '\'', ..] => Some(after(3)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_both_patterns_outside_comments_and_strings() {
        let source = "static mut A: u8 = 0;\n\
                      fn f(x: &'static mut u8) {}\n\
                      // static mut in a comment\n\
                      let s = \"static mut in a string\";\n\
                      let t = \"a string that\n\
                      static mut spans lines\";\n\
                      static   mut\tB: u8 = 0;\n";
        assert_eq!(
            scan("kernel/src/lib.rs", source),
            [
                "kernel/src/lib.rs:1: static mut",
                "kernel/src/lib.rs:2: &'static mut",
                "kernel/src/lib.rs:7: static mut",
            ]
        );
    }

    // Each of these once hid the line after it, or would have: a quote
    // the scanner took for the start of a string.
    #[test]
    fn finds_patterns_after_quotes_in_other_literals() {
        let source = r##"let q = '"';
static mut A: u8 = 0;
let b = b'"';
static mut B: u8 = 0;
let e = '\'';
static mut C: u8 = 0;
let r = r#"a "quoted" static mut"#;
static mut D: u8 = 0;
let s = "an \" escaped quote";
static mut E: u8 = 0;
/* a block comment with a " and /* a nested one */ static mut */
static mut F: u8 = 0;
fn f<'a>(x: &'a u8, y: &'static mut u8) {}
let c = cr"a\";
static mut G: u8 = 0;
let d = cr#"a "quoted" static mut"#;
static mut H: u8 = 0;
"##;
        assert_eq!(
            scan("kernel/src/lib.rs", source),
            [
                "kernel/src/lib.rs:2: static mut",
                "kernel/src/lib.rs:4: static mut",
                "kernel/src/lib.rs:6: static mut",
                "kernel/src/lib.rs:8: static mut",
                "kernel/src/lib.rs:10: static mut",
                "kernel/src/lib.rs:12: static mut",
                "kernel/src/lib.rs:13: &'static mut",
                "kernel/src/lib.rs:15: static mut",
                "kernel/src/lib.rs:17: static mut",
            ]
        );
    }

    // rustfmt joins these lines, but not under rustfmt::skip.
    #[test]
    fn finds_patterns_split_across_lines() {
        let source = "#[rustfmt::skip]\n\
                      static\n\
                      mut A: u8 = 0;\n\
                      fn f(x: &'static // a comment\n\
                      \x20   mut u8) {}\n";
        assert_eq!(
            scan("kernel/src/lib.rs", source),
            [
                "kernel/src/lib.rs:2: static mut",
                "kernel/src/lib.rs:4: &'static mut",
            ]
        );
    }

    #[test]
    fn skips_a_listed_file_that_is_gone() {
        assert_eq!(source("kernel/src/no_such_file.rs").unwrap(), None);
    }

    #[test]
    fn allows_the_loader_entry_only() {
        let source = "boot: &'static mut bootloader_api::BootInfo,\n\
                      other: &'static mut u8,\n\
                      split: &'static\n\
                      mut bootloader_api::BootInfo,\n";
        assert_eq!(
            scan("arch/x86_64/src/lib.rs", source),
            [
                "arch/x86_64/src/lib.rs:2: &'static mut",
                "arch/x86_64/src/lib.rs:3: &'static mut",
            ]
        );
        assert_eq!(scan("kernel/src/lib.rs", source).len(), 3);
    }

    #[test]
    fn ignores_names_that_only_contain_the_words() {
        assert!(scan("a.rs", "let static_mutex = mut_static;\n").is_empty());
    }
}

//! Asking `lsblk` about block devices.

use std::process::Command;

use anyhow::Result;

use crate::cmd::output;

/// One line of `lsblk -P` output: a device and the columns asked for.
pub struct Row(Vec<(String, String)>);

impl Row {
    /// Splits a line of `lsblk -P` output, `KEY="value" ...`. lsblk escapes
    /// quotes inside values as `\x22`, so the next quote always ends a value.
    fn parse(line: &str) -> Self {
        let mut pairs = Vec::new();
        let mut rest = line;
        while let Some((key, after)) = rest.split_once("=\"") {
            let Some((value, after)) = after.split_once('"') else {
                break;
            };
            pairs.push((key.trim().to_owned(), value.to_owned()));
            rest = after;
        }
        Self(pairs)
    }

    /// The value of column `key`, or "" when lsblk left it out or empty.
    pub fn get(&self, key: &str) -> &str {
        self.0.iter().find(|(k, _)| k == key).map_or("", |(_, v)| v)
    }
}

/// Runs `lsblk -P` with `args`, one row per device.
pub fn lsblk(args: &[&str]) -> Result<Vec<Row>> {
    let list = output(Command::new("lsblk").arg("-P").args(args))?;
    Ok(list.lines().map(Row::parse).collect())
}

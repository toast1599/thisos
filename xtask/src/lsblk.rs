//! Asking `lsblk` about block devices.

use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::cmd::output;

/// One device in `lsblk -J` output, with the columns asked for. Column
/// names are lsblk's, in lower case.
pub struct Row(Value);

impl Row {
    /// The text in column `key`, or "" when lsblk left it out or null.
    pub fn text(&self, key: &str) -> &str {
        self.0[key].as_str().unwrap_or("")
    }

    /// Whether column `key` is true.
    pub fn flag(&self, key: &str) -> bool {
        self.0[key].as_bool() == Some(true)
    }

    /// The number in column `key`, when there is one.
    pub fn number(&self, key: &str) -> Option<u64> {
        self.0[key].as_u64()
    }
}

/// Runs `lsblk -J --list` with `args`, one row per device.
pub fn lsblk(args: &[&str]) -> Result<Vec<Row>> {
    let text = output(Command::new("lsblk").args(["-J", "--list"]).args(args))?;
    parse(&text)
}

/// Splits `lsblk -J` output into rows.
pub fn parse(text: &str) -> Result<Vec<Row>> {
    let mut json: Value = serde_json::from_str(text).context("parsing lsblk's JSON")?;
    let Value::Array(rows) = json["blockdevices"].take() else {
        bail!("lsblk's JSON has no blockdevices list: {text}");
    };
    Ok(rows.into_iter().map(Row).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_columns_by_type() {
        let rows = parse(
            r#"{"blockdevices": [
                {"path": "/dev/sdb1", "mountpoint": "/media/bob/THISOS BOOT", "rm": true, "size": 4096},
                {"path": "/dev/sdb2", "mountpoint": null, "rm": false, "size": null}
            ]}"#,
        )
        .unwrap();
        assert_eq!(rows[0].text("mountpoint"), "/media/bob/THISOS BOOT");
        assert!(rows[0].flag("rm"));
        assert_eq!(rows[0].number("size"), Some(4096));
        assert_eq!(rows[1].text("mountpoint"), "");
        assert!(!rows[1].flag("rm"));
        assert_eq!(rows[1].number("size"), None);
        assert_eq!(rows[1].text("serial"), "");
    }
}

//! Writing the image to the USB stick for a laptop run.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::build::{self, BUILD_FILE, Kernel};
use crate::cmd::{confirm, exec, output, output_bytes};
use crate::lsblk::{Row, lsblk};

/// A whole disk, as `lsblk` describes it.
#[derive(Debug)]
struct Disk {
    path: String,
    /// The disk's `/dev/disk/by-id` link, when udev made one. It names the
    /// disk by model and serial, so it disappears with the stick instead of
    /// moving to whatever disk takes the stick's `/dev/sdX` next.
    by_id: Option<String>,
    size: u64,
    model: String,
    serial: String,
}

impl fmt::Display for Disk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let gib = self.size as f64 / f64::from(1u32 << 30);
        write!(
            f,
            "{}: {} {}, {gib:.1} GiB",
            self.path, self.model, self.serial
        )
    }
}

/// Builds the image of `kernel`, writes it to a USB stick with `sudo dd`,
/// reads it back to check it, moves the backup GPT to the end of the stick
/// and powers the stick off.
pub fn flash(release: bool, kernel: Kernel, device: Option<&str>) -> Result<()> {
    let image = build::build(release, kernel)?;
    let data =
        fs::read(&image.path).with_context(|| format!("reading {}", image.path.display()))?;

    let disks = usb_disks(lsblk(&[
        "-d",
        "-b",
        "-e7",
        "-o",
        "PATH,TYPE,TRAN,RM,HOTPLUG,SIZE,MODEL,SERIAL,ID-LINK",
    ])?)?;
    // From here on every command names the disk by its by-id link, so a
    // stick swapped while the question below waits makes them fail.
    let (disk, dev) = pick(&disks, device)?;
    ensure!(
        data.len() as u64 <= disk.size,
        "the image is {} bytes, larger than {disk}",
        data.len()
    );

    println!("image: {} ({} bytes)", image.path.display(), data.len());
    println!("       {}", image.id);
    println!("stick: {disk}");
    println!("       {dev}");
    let held = match stick_build(dev) {
        Ok(Some(held)) if held == image.id => "this same build".into(),
        Ok(Some(held)) => held,
        Ok(None) => "no ThisOS build ID".into(),
        // Not being able to read it is no reason to refuse to overwrite it.
        Err(e) => format!("unknown ({e:#})"),
    };
    println!("stick holds: {held}");
    // Always asked: external USB disks report hotplug too, so this is what
    // tells the stick from a backup drive left plugged in.
    if !confirm(&format!(
        "Erase everything on {} and write the image? [y/N] ",
        disk.path
    ))? {
        bail!("canceled; nothing was written");
    }

    unmount(dev)?;
    // Ask for the password now, not in the middle of dd's progress output.
    exec(Command::new("sudo").arg("-v"))?;
    // nocreat, because a stick pulled during the password prompt takes its
    // link with it, and dd would otherwise write the image to a new file there.
    exec(Command::new("sudo").args([
        "dd",
        &format!("if={}", image.path.display()),
        &format!("of={dev}"),
        "bs=4M",
        "conv=fsync,nocreat",
        "status=progress",
    ]))?;
    // This has to come before sgdisk -e, which rewrites the primary GPT header.
    verify(dev, &data)?;
    exec(Command::new("sudo").args(["sgdisk", "-e", dev]))?;
    let check = output(Command::new("sudo").args(["sgdisk", "-v", dev]))?;
    ensure!(
        check.contains("No problems found"),
        "sgdisk -v found problems on {dev}:\n{check}"
    );
    // The desktop may have mounted the new partition after dd.
    unmount(dev)?;
    exec(Command::new("udisksctl").args(["power-off", "-b", dev]))?;
    println!("{} written, verified and powered off", disk.path);
    Ok(())
}

/// The whole disks on USB that are removable or hotplug, from lsblk's rows
/// for every disk: the only disks `flash` writes.
fn usb_disks(rows: Vec<Row>) -> Result<Vec<Disk>> {
    let mut disks = Vec::new();
    for row in rows {
        let removable = row.flag("rm") || row.flag("hotplug");
        if row.text("type") != "disk" || row.text("tran") != "usb" || !removable {
            continue;
        }
        let link = row.text("id-link");
        disks.push(Disk {
            path: row.text("path").into(),
            by_id: (!link.is_empty()).then(|| format!("/dev/disk/by-id/{link}")),
            size: row
                .number("size")
                .with_context(|| format!("lsblk gave no size for {}", row.text("path")))?,
            model: row.text("model").into(),
            serial: row.text("serial").into(),
        });
    }
    Ok(disks)
}

/// The disk to write: the one `device` names, or else the only one. Returns
/// it with its by-id path.
fn pick<'a>(disks: &'a [Disk], device: Option<&str>) -> Result<(&'a Disk, &'a str)> {
    let disk = match (device, disks) {
        (Some(path), _) => disks
            .iter()
            .find(|disk| disk.path == path)
            .with_context(|| {
                format!("{path} isn't a whole disk on USB that is removable or hotplug")
            })?,
        (None, [disk]) => disk,
        (None, []) => bail!("no removable USB disk found"),
        (None, _) => {
            let list: Vec<String> = disks.iter().map(Disk::to_string).collect();
            bail!(
                "more than one USB disk; pick one with --device:\n  {}",
                list.join("\n  ")
            );
        }
    };
    let dev = disk.by_id.as_deref().with_context(|| {
        format!(
            "{disk} has no /dev/disk/by-id link, so a disk swapped in for it couldn't be told apart"
        )
    })?;
    Ok((disk, dev))
}

/// Reads the build ID from the first FAT partition on `device`, mounting it
/// for the read when it isn't mounted already.
fn stick_build(device: &str) -> Result<Option<String>> {
    let rows = lsblk(&["-o", "PATH,TYPE,FSTYPE", device])?;
    let Some(partition) = rows
        .iter()
        .find(|row| row.text("type") == "part" && row.text("fstype") == "vfat")
    else {
        return Ok(None);
    };
    let path = partition.text("path");
    let mut dir = mountpoint(path)?;
    let mounted_here = dir.is_empty();
    if mounted_here {
        output(Command::new("udisksctl").args(["mount", "-b", path]))?;
        dir = mountpoint(path)?;
        ensure!(
            !dir.is_empty(),
            "udisksctl mounted {path}, but lsblk shows no mount point for it"
        );
    }
    let file = Path::new(&dir).join(BUILD_FILE);
    let held = match fs::read_to_string(&file) {
        Ok(text) => Ok(Some(text.trim().to_owned())),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", file.display())),
    };
    if mounted_here {
        output(Command::new("udisksctl").args(["unmount", "-b", path]))?;
    }
    held
}

fn mountpoint(partition: &str) -> Result<String> {
    let rows = lsblk(&["-o", "MOUNTPOINT", partition])?;
    Ok(rows
        .first()
        .map_or("", |row| row.text("mountpoint"))
        .to_owned())
}

/// Unmounts every mounted partition on `device`.
fn unmount(device: &str) -> Result<()> {
    for row in lsblk(&["-o", "PATH,MOUNTPOINT", device])? {
        if !row.text("mountpoint").is_empty() {
            exec(Command::new("udisksctl").args(["unmount", "-b", row.text("path")]))?;
        }
    }
    Ok(())
}

/// Reads the image's length back from `device` and compares it with `image`.
fn verify(device: &str, image: &[u8]) -> Result<()> {
    // Dropping the device's cached pages makes the read come from the stick
    // and not from what dd left in memory. O_DIRECT would do the same, but
    // uutils dd, which Ubuntu ships as dd, fails with EINVAL reading a block
    // device that way.
    exec(Command::new("sudo").args(["blockdev", "--flushbufs", device]))?;
    let read = output_bytes(Command::new("sudo").args([
        "dd",
        &format!("if={device}"),
        "bs=4M",
        "iflag=count_bytes",
        &format!("count={}", image.len()),
        "status=none",
    ]))?;
    if let Some(offset) = read.iter().zip(image).position(|(a, b)| a != b) {
        bail!("{device} differs from the image at byte {offset}");
    }
    ensure!(
        read.len() >= image.len(),
        "read back {} of the image's {} bytes from {device}",
        read.len(),
        image.len()
    );
    println!("verified: {device} holds the image");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsblk::parse;

    // The internal NVMe disk, a USB stick, a USB SSD that reports only
    // hotplug, a fixed USB disk, and a stick with no by-id link.
    const DISKS: &str = r#"{"blockdevices": [
        {"path": "/dev/nvme0n1", "type": "disk", "tran": "nvme", "rm": false, "hotplug": false,
         "size": 1024209543168, "model": "WD PC SN740", "serial": "24081A800209",
         "id-link": "nvme-eui.e8238fa6bf530001001b448b475a3fd7"},
        {"path": "/dev/sdb", "type": "disk", "tran": "usb", "rm": true, "hotplug": true,
         "size": 30765219840, "model": "Ultra", "serial": "4C530001",
         "id-link": "usb-SanDisk_Ultra_4C530001-0:0"},
        {"path": "/dev/sdc", "type": "disk", "tran": "usb", "rm": false, "hotplug": true,
         "size": 2000398934016, "model": "Portable SSD", "serial": "S6XNNS0T",
         "id-link": "usb-Samsung_Portable_SSD_S6XNNS0T-0:0"},
        {"path": "/dev/sdd", "type": "disk", "tran": "usb", "rm": false, "hotplug": false,
         "size": 500107862016, "model": "Fixed", "serial": "F1", "id-link": "usb-Fixed_F1-0:0"},
        {"path": "/dev/sde", "type": "disk", "tran": "usb", "rm": true, "hotplug": true,
         "size": 8053063680, "model": "Flash Disk", "serial": null, "id-link": null}
    ]}"#;

    fn disks() -> Vec<Disk> {
        usb_disks(parse(DISKS).unwrap()).unwrap()
    }

    #[test]
    fn keeps_only_removable_or_hotplug_usb_disks() {
        let paths: Vec<String> = disks().into_iter().map(|disk| disk.path).collect();
        assert_eq!(paths, ["/dev/sdb", "/dev/sdc", "/dev/sde"]);
    }

    #[test]
    fn picks_by_device_and_returns_the_by_id_path() {
        let disks = disks();
        let (disk, dev) = pick(&disks, Some("/dev/sdb")).unwrap();
        assert_eq!(disk.path, "/dev/sdb");
        assert_eq!(dev, "/dev/disk/by-id/usb-SanDisk_Ultra_4C530001-0:0");
        assert!(pick(&disks, Some("/dev/nvme0n1")).is_err());
        assert!(pick(&disks, Some("/dev/sdd")).is_err());
    }

    #[test]
    fn refuses_to_guess() {
        let disks = disks();
        let several = pick(&disks, None).unwrap_err().to_string();
        assert!(
            several.contains("/dev/sdb") && several.contains("/dev/sdc"),
            "{several}"
        );
        assert!(pick(&[], None).is_err());
        assert!(pick(&disks, Some("/dev/sde")).is_err(), "no by-id link");
    }
}

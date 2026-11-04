//! Writing the image to the USB stick for a laptop run.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail, ensure};

use crate::build::{self, BUILD_FILE};
use crate::cmd::{confirm, exec, output, output_bytes};
use crate::lsblk::lsblk;

/// A whole disk, as `lsblk` describes it.
struct Disk {
    path: String,
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

/// Builds the image, writes it to a USB stick with `sudo dd`, reads it back
/// to check it, moves the backup GPT to the end of the stick and powers the
/// stick off.
pub fn flash(release: bool, device: Option<&str>, yes: bool) -> Result<()> {
    let image = build::build(release)?;
    let data =
        fs::read(&image.path).with_context(|| format!("reading {}", image.path.display()))?;

    let disks = usb_disks()?;
    let disk = match (device, disks.as_slice()) {
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
    ensure!(
        data.len() as u64 <= disk.size,
        "the image is {} bytes, larger than {disk}",
        data.len()
    );

    println!("image: {} ({} bytes)", image.path.display(), data.len());
    println!("       {}", image.id);
    println!("stick: {disk}");
    let held = match stick_build(&disk.path) {
        Ok(Some(held)) if held == image.id => "this same build".into(),
        Ok(Some(held)) => held,
        Ok(None) => "no ThisOS build ID".into(),
        // Not being able to read it is no reason to refuse to overwrite it.
        Err(e) => format!("unknown ({e:#})"),
    };
    println!("stick holds: {held}");
    if !yes
        && !confirm(&format!(
            "Erase everything on {} and write the image? [y/N] ",
            disk.path
        ))?
    {
        bail!("canceled; nothing was written");
    }

    unmount(&disk.path)?;
    // Ask for the password now, not in the middle of dd's progress output.
    exec(Command::new("sudo").arg("-v"))?;
    exec(Command::new("sudo").args([
        "dd",
        &format!("if={}", image.path.display()),
        &format!("of={}", disk.path),
        "bs=4M",
        "conv=fsync",
        "status=progress",
    ]))?;
    // This has to come before sgdisk -e, which rewrites the primary GPT header.
    verify(&disk.path, &data)?;
    exec(Command::new("sudo").args(["sgdisk", "-e", &disk.path]))?;
    let check = output(Command::new("sudo").args(["sgdisk", "-v", &disk.path]))?;
    ensure!(
        check.contains("No problems found"),
        "sgdisk -v found problems on {}:\n{check}",
        disk.path
    );
    // The desktop may have mounted the new partition after dd.
    unmount(&disk.path)?;
    exec(Command::new("udisksctl").args(["power-off", "-b", &disk.path]))?;
    println!("{} written, verified and powered off", disk.path);
    Ok(())
}

/// Whole disks on USB that are removable or hotplug: the only disks `flash`
/// writes.
fn usb_disks() -> Result<Vec<Disk>> {
    let rows = lsblk(&[
        "-d",
        "-b",
        "-e7",
        "-o",
        "PATH,TYPE,TRAN,RM,HOTPLUG,SIZE,MODEL,SERIAL",
    ])?;
    let mut disks = Vec::new();
    for row in rows {
        let removable = row.get("RM") == "1" || row.get("HOTPLUG") == "1";
        if row.get("TYPE") != "disk" || row.get("TRAN") != "usb" || !removable {
            continue;
        }
        disks.push(Disk {
            path: row.get("PATH").into(),
            size: row
                .get("SIZE")
                .parse()
                .with_context(|| format!("lsblk size {:?}", row.get("SIZE")))?,
            model: row.get("MODEL").into(),
            serial: row.get("SERIAL").into(),
        });
    }
    Ok(disks)
}

/// Reads the build ID from the first FAT partition on `device`, mounting it
/// for the read when it isn't mounted already.
fn stick_build(device: &str) -> Result<Option<String>> {
    let rows = lsblk(&["-n", "-o", "PATH,TYPE,FSTYPE", device])?;
    let Some(partition) = rows
        .iter()
        .find(|row| row.get("TYPE") == "part" && row.get("FSTYPE") == "vfat")
    else {
        return Ok(None);
    };
    let path = partition.get("PATH");
    let mut dir = mountpoint(path)?;
    let mounted_here = dir.is_empty();
    if mounted_here {
        output(Command::new("udisksctl").args(["mount", "-b", path]))?;
        dir = mountpoint(path)?;
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
    let rows = lsblk(&["-n", "-o", "MOUNTPOINT", partition])?;
    Ok(rows
        .first()
        .map_or("", |row| row.get("MOUNTPOINT"))
        .to_owned())
}

/// Unmounts every mounted partition on `device`.
fn unmount(device: &str) -> Result<()> {
    for row in lsblk(&["-n", "-o", "PATH,MOUNTPOINT", device])? {
        if !row.get("MOUNTPOINT").is_empty() {
            exec(Command::new("udisksctl").args(["unmount", "-b", row.get("PATH")]))?;
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

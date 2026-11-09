//! The Limine boot loader and the aarch64 disk image it boots from.

use std::fs::{self, OpenOptions};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};

use crate::build::BUILD_FILE;
use crate::root;

const VERSION: &str = "12.9.6";

/// The SHA-256 of the release's `limine-binary.tar.xz`. Bumping `VERSION`
/// means downloading the new archive and checking it before changing this.
const SHA256: &str = "939f670e0831cc940453fd3a0ddfb2b324c0b4e3c5d0e741c190aa0d13df0103";

/// The archive is under 1 MiB; this only stops a runaway download.
const MAX_DOWNLOAD: u64 = 16 << 20;

const CONFIG: &str = "\
timeout: 0

/ThisOS
    protocol: limine
    path: boot():/thisos-kernel
";

/// Writes a GPT disk image to `path` with one EFI system partition holding
/// Limine, its config, the kernel ELF at `elf` and, when there is one, the
/// build ID.
pub fn write_image(elf: &Path, id: Option<&str>, path: &Path) -> Result<()> {
    let efi = loader()?;
    let mut files = vec![
        (
            "EFI/BOOT/BOOTAA64.EFI",
            fs::read(&efi).with_context(|| format!("reading {}", efi.display()))?,
        ),
        ("EFI/BOOT/limine.conf", CONFIG.as_bytes().to_vec()),
        (
            "thisos-kernel",
            fs::read(elf).with_context(|| format!("reading {}", elf.display()))?,
        ),
    ];
    if let Some(id) = id {
        files.push((BUILD_FILE, format!("{id}\n").into_bytes()));
    }
    let fat = fat(&files)?;
    gpt(&fat, path).with_context(|| format!("writing {}", path.display()))
}

/// `BOOTAA64.EFI` from the pinned release, which the first call downloads
/// into `target/limine/<version>/`.
fn loader() -> Result<PathBuf> {
    let dir = root().join("target/limine").join(VERSION);
    let efi = dir.join("BOOTAA64.EFI");
    if efi.is_file() {
        return Ok(efi);
    }

    let url = format!(
        "https://github.com/limine-bootloader/limine/releases/download/v{VERSION}/limine-binary.tar.xz"
    );
    println!("downloading {url}");
    let mut data = Vec::new();
    ureq::get(&url)
        .call()
        .with_context(|| format!("downloading {url}"))?
        .into_body()
        .into_reader()
        .take(MAX_DOWNLOAD)
        .read_to_end(&mut data)
        .with_context(|| format!("downloading {url}"))?;
    let hash: String = Sha256::digest(&data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    ensure!(
        hash == SHA256,
        "{url} has SHA-256 {hash}, but xtask pins {SHA256}"
    );

    let mut tar = Vec::new();
    lzma_rs::xz_decompress(&mut Cursor::new(&data), &mut tar)
        .with_context(|| format!("decompressing {url}"))?;
    let mut archive = tar::Archive::new(Cursor::new(tar));
    for entry in archive.entries().context("reading the Limine archive")? {
        let mut entry = entry.context("reading the Limine archive")?;
        if entry.path().context("reading the Limine archive")?
            != Path::new("limine-binary/BOOTAA64.EFI")
        {
            continue;
        }
        fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        // Unpacked under another name first, so that an interrupted unpack
        // isn't mistaken for a cached loader next time.
        let part = dir.join("BOOTAA64.EFI.part");
        entry
            .unpack(&part)
            .with_context(|| format!("unpacking {}", part.display()))?;
        fs::rename(&part, &efi).with_context(|| format!("renaming {}", part.display()))?;
        return Ok(efi);
    }
    bail!("{url} has no limine-binary/BOOTAA64.EFI")
}

/// A FAT filesystem holding `files`, by their paths from its root.
fn fat(files: &[(&str, Vec<u8>)]) -> Result<Vec<u8>> {
    const MIB: usize = 1 << 20;
    let contents: usize = files.iter().map(|(_, data)| data.len()).sum();
    // Room for the FAT, directories and cluster rounding.
    let mut disk = Cursor::new(vec![0; contents.div_ceil(MIB) * MIB + 2 * MIB]);
    fatfs::format_volume(
        &mut disk,
        fatfs::FormatVolumeOptions::new().volume_label(*b"THISOS     "),
    )
    .context("formatting the FAT filesystem")?;
    {
        let fs = fatfs::FileSystem::new(&mut disk, fatfs::FsOptions::new())
            .context("opening the FAT filesystem")?;
        let root = fs.root_dir();
        for (path, data) in files {
            // create_dir opens a directory that exists, and makes only the
            // last one in a path, so each ancestor goes first.
            for (end, _) in path.match_indices('/') {
                root.create_dir(&path[..end])
                    .with_context(|| format!("creating {}", &path[..end]))?;
            }
            root.create_file(path)
                .and_then(|mut file| file.write_all(data))
                .with_context(|| format!("writing {path}"))?;
        }
    }
    Ok(disk.into_inner())
}

/// Writes a GPT disk to `path` whose one partition, the EFI system
/// partition, holds `fat`.
fn gpt(fat: &[u8], path: &Path) -> Result<()> {
    let mut disk = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(path)?;
    let partition = u64::try_from(fat.len())?;
    // The primary and backup headers and partition tables.
    let size = partition + (64 << 10);
    disk.set_len(size)?;

    let blocks = u32::try_from(size / 512 - 1).unwrap_or(u32::MAX);
    gpt::mbr::ProtectiveMBR::with_lb_size(blocks)
        .overwrite_lba0(&mut disk)
        .context("writing the protective MBR")?;
    let block = gpt::disk::LogicalBlockSize::Lb512;
    let mut table = gpt::GptConfig::new()
        .writable(true)
        .initialized(false)
        .logical_block_size(block)
        .create_from_device(Box::new(&mut disk), None)
        .context("creating the GPT")?;
    table
        .update_partitions(Default::default())
        .context("creating the GPT")?;
    let id = table
        .add_partition("boot", partition, gpt::partition_types::EFI, 0, None)
        .context("adding the EFI system partition")?;
    let start = table
        .partitions()
        .get(&id)
        .context("the EFI system partition vanished")?
        .bytes_start(block)
        .context("finding the EFI system partition")?;
    table.write().context("writing the GPT")?;

    disk.seek(SeekFrom::Start(start))?;
    disk.write_all(fat)?;
    Ok(())
}

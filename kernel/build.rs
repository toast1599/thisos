//! Links the aarch64 kernel where Limine loads it. The x86_64 kernel needs
//! nothing here: `bootloader` loads any position-independent ELF.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("aarch64") {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/../arch/aarch64/linker.ld");
        println!("cargo::rustc-link-arg=-T{script}");
        println!("cargo::rerun-if-changed={script}");
    }
}

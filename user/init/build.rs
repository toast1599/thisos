//! Links init at a fixed address, lld's default base for x86_64, so that
//! the kernel loads it without relocating it.

fn main() {
    println!("cargo::rustc-link-arg-bins=--no-pie");
}

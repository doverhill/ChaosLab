use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let asm_source = manifest_dir.join("src/arch/x86_64/asm/ap_trampoline.asm");
    let trampoline_binary = out_dir.join("ap_trampoline.bin");

    // assemble the AP trampoline to a flat binary
    let status = Command::new("nasm")
        .args([
            "-f", "bin",
            "-o", trampoline_binary.to_str().unwrap(),
            asm_source.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nasm. Is nasm installed?");

    if !status.success() {
        panic!("nasm failed to assemble ap_trampoline.asm");
    }

    println!("cargo:rerun-if-changed=src/arch/x86_64/asm/ap_trampoline.asm");
}

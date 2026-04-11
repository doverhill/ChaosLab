use bootloader::DiskImageBuilder;
use std::process::Command;
use std::{env, path::PathBuf};

/// Find llvm-objcopy from the Rust toolchain's sysroot.
fn find_llvm_objcopy() -> Option<PathBuf> {
    let output = Command::new("rustc").arg("--print").arg("sysroot").output().ok()?;
    let sysroot = PathBuf::from(String::from_utf8(output.stdout).ok()?.trim().to_string());
    // Search all host triples for the binary
    for entry in std::fs::read_dir(sysroot.join("lib/rustlib")).ok()? {
        let path = entry.ok()?.path().join("bin/llvm-objcopy");
        if path.exists() { return Some(path); }
    }
    None
}

/// Strip debug info from an ELF and copy to the staging directory.
fn strip_and_stage(source: &str, destination: &PathBuf) {
    if let Some(objcopy) = find_llvm_objcopy() {
        let status = Command::new(&objcopy)
            .arg("--strip-debug")
            .arg(source)
            .arg(destination)
            .status()
            .expect("Failed to run llvm-objcopy");
        if status.success() { return; }
    }
    eprintln!("cargo:warning=llvm-objcopy not found, using unstripped ELFs");
    std::fs::copy(source, destination).unwrap();
}

fn main() {
    // set by cargo for the storm kernel artifact dependency
    let kernel_path = env::var("CARGO_BIN_FILE_STORM").unwrap();
    println!("cargo:rustc-env=KERNEL_IMAGE={}", kernel_path);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ---- Build ramdisk tar from all application ELFs ----
    let test_app_path = env::var("CARGO_BIN_FILE_TEST_APP").unwrap();

    // Strip debug info and stage with clean names for the tar entries.
    // Debug sections are ~95% of the ELF size — stripping reduces a
    // 1 MiB "hello world" app to ~64 KiB.
    let staging_dir = out_dir.join("ramdisk_staging");
    let _ = std::fs::create_dir_all(&staging_dir);
    strip_and_stage(&test_app_path, &staging_dir.join("test_app_1.elf"));
    strip_and_stage(&test_app_path, &staging_dir.join("test_app_2.elf"));

    let ramdisk_path = out_dir.join("ramdisk.tar");
    let status = Command::new("tar")
        .arg("--format").arg("ustar")  // plain ustar — no PAX extensions
        .arg("-cf")
        .arg(&ramdisk_path)
        .arg("-C").arg(&staging_dir)
        .arg("test_app_1.elf")
        .arg("test_app_2.elf")
        .env("COPYFILE_DISABLE", "1")  // suppress macOS resource fork files
        .status()
        .expect("Failed to run tar. Is tar installed?");
    if !status.success() {
        panic!("tar failed to create ramdisk");
    }
    println!("cargo:warning=Ramdisk: {}", ramdisk_path.display());

    // ---- Build disk images ----
    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));
    disk_builder.set_ramdisk(ramdisk_path);

    let uefi_path = out_dir.join("chaos-uefi.img");
    let bios_path = out_dir.join("chaos-bios.img");

    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}

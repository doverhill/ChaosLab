use bootloader::DiskImageBuilder;
use std::process::Command;
use std::{env, path::PathBuf};

fn main() {
    // set by cargo for the storm kernel artifact dependency
    let kernel_path = env::var("CARGO_BIN_FILE_STORM").unwrap();
    println!("cargo:rustc-env=KERNEL_IMAGE={}", kernel_path);

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ---- Build ramdisk tar from all application ELFs ----
    let test_app_path = env::var("CARGO_BIN_FILE_TEST_APP").unwrap();

    let ramdisk_path = out_dir.join("ramdisk.tar");
    let status = Command::new("tar")
        .arg("--format").arg("ustar")  // plain ustar — no PAX extensions
        .arg("-cf")
        .arg(&ramdisk_path)
        .arg("-C").arg(PathBuf::from(&test_app_path).parent().unwrap())
        .arg(PathBuf::from(&test_app_path).file_name().unwrap())
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

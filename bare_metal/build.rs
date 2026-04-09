use bootloader::DiskImageBuilder;
use std::{env, path::PathBuf};

fn main() {
    // set by cargo for the storm kernel artifact dependency
    let kernel_path = env::var("CARGO_BIN_FILE_STORM").unwrap();
    println!("cargo:rustc-env=KERNEL_IMAGE={}", kernel_path);

    let mut disk_builder = DiskImageBuilder::new(PathBuf::from(kernel_path));

    // bundle the test app as a ramdisk so the kernel can load it
    let test_app_path = env::var("CARGO_BIN_FILE_TEST_APP").unwrap();
    println!("cargo:rustc-env=TEST_APP_IMAGE={}", test_app_path);
    disk_builder.set_ramdisk(PathBuf::from(test_app_path));

    // specify output paths
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let uefi_path = out_dir.join("chaos-uefi.img");
    let bios_path = out_dir.join("chaos-bios.img");

    // create the disk images
    disk_builder.create_uefi_image(&uefi_path).unwrap();
    disk_builder.create_bios_image(&bios_path).unwrap();

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", bios_path.display());
}

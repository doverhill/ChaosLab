use std::{
    env,
    process::{self, Command},
};

fn main() {
    let kernel_image = env!("KERNEL_IMAGE");
    println!("Running in UEFI mode using kernel image {}", kernel_image);

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    qemu.arg("-smp").arg("4");
    qemu.arg("-accel").arg("whpx");
    qemu.arg("-serial").arg("stdio");
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}

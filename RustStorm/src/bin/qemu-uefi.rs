use std::{
    env,
    process::{self, Command},
};

fn main() {
    let kernel_image = env!("KERNEL_IMAGE");
    println!("Running in UEFI mode using kernel {}", kernel_image);
    println!("Disk image {}", env!("UEFI_IMAGE"));

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    qemu.arg("-smp").arg("4");
    qemu.arg("-m").arg("64");
    // qemu.arg("-accel").arg("whpx");
    qemu.arg("-serial").arg("stdio");
//    qemu.arg("-d").arg("mmu,unimp,guest_errors"); //int,vpu,
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}

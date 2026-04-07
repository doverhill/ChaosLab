use std::{
    env,
    process::{self, Command},
};

fn main() {
    let kernel_image = env!("KERNEL_IMAGE");
    println!("Running in UEFI mode using kernel {}", kernel_image);
    println!("Disk image {}", env!("UEFI_IMAGE"));

    let ovmf_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/ovmf");
    let prebuilt = ovmf_prebuilt::Prebuilt::fetch(ovmf_prebuilt::Source::LATEST, &ovmf_dir).expect("failed to fetch OVMF prebuilt");
    let ovmf_code = prebuilt.get_file(ovmf_prebuilt::Arch::X64, ovmf_prebuilt::FileType::Code);
    let ovmf_vars = prebuilt.get_file(ovmf_prebuilt::Arch::X64, ovmf_prebuilt::FileType::Vars);

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,readonly=on,file={}", ovmf_code.display()));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,file={}", ovmf_vars.display()));
    qemu.arg("-smp").arg("4");
    qemu.arg("-m").arg("64");
    // qemu.arg("-accel").arg("whpx");
    qemu.arg("-serial").arg("stdio");
//    qemu.arg("-d").arg("mmu,unimp,guest_errors"); //int,vpu,
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}

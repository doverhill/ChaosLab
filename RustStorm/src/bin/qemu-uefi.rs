use std::{
    env, io::Write,
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

    let monitor_sock = "/tmp/qemu-chaoslab-monitor.sock";
    let screenshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/screenshot.ppm");
    let _ = std::fs::remove_file(monitor_sock);

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive").arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,readonly=on,file={}", ovmf_code.display()));
    qemu.arg("-drive").arg(format!("if=pflash,format=raw,file={}", ovmf_vars.display()));
    qemu.arg("-vga").arg("none");
    qemu.arg("-device").arg("VGA,vgamem_mb=8,xres=800,yres=600");
    qemu.arg("-smp").arg("4");
    qemu.arg("-m").arg("64");
    qemu.arg("-serial").arg("stdio");
    qemu.arg("-display").arg("cocoa,full-screen=on,zoom-to-fit=on");
    qemu.arg("-device").arg("isa-debug-exit,iobase=0xf4,iosize=0x01");
    qemu.arg("-no-reboot");  // exit on triple fault instead of rebooting
    qemu.arg("-monitor").arg(format!("unix:{},server,nowait", monitor_sock));
//    qemu.arg("-d").arg("mmu,unimp,guest_errors"); //int,vpu,

    // spawn so we can take a screenshot while QEMU runs
    let mut child = qemu.spawn().unwrap();

    // background thread: wait for kernel to boot, then take a screenshot via QEMU monitor
    let sock_path = monitor_sock.to_string();
    let img_path = screenshot_path.display().to_string();
    std::thread::spawn(move || {
        // give the kernel time to boot and render to framebuffer
        std::thread::sleep(std::time::Duration::from_secs(8));
        if let Ok(mut stream) = std::os::unix::net::UnixStream::connect(&sock_path) {
            let cmd = format!("screendump {}\n", img_path);
            let _ = stream.write_all(cmd.as_bytes());
            std::thread::sleep(std::time::Duration::from_millis(500));
            eprintln!("Screenshot saved to {}", img_path);
        }
    });

    let exit_status = child.wait().unwrap();
    let _ = std::fs::remove_file(monitor_sock);

    // isa-debug-exit: QEMU exit code = (value << 1) | 1, so kernel writing 0 → exit 1 (success)
    let code = exit_status.code().unwrap_or(-1);
    process::exit(if code == 1 { 0 } else { code });
}

#![no_std]
#![no_main]

mod vga_buffer;

/// This is the entry point of our system
#[no_mangle]
pub extern "C" fn _start() -> ! {
    use core::fmt::Write;
    vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    write!(
        vga_buffer::WRITER.lock(),
        ", some test numbers: {} {}",
        42,
        1.337
    )
    .unwrap();

    loop {}
}

/// This is the panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

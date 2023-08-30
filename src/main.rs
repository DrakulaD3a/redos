#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rudos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use rudos::println;

entry_point!(kernel_main);

/// This is the typechecked entry point of our system
fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");
    rudos::init();

    // We need to manually call this because we are in no_main project
    #[cfg(test)]
    test_main();

    rudos::hlt_loop();
}

/// This is the panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    rudos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rudos::test_panic_handler(info)
}

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rudos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use rudos::println;

/// This is the entry point of our system
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    rudos::init();

    // We need to manually call this because we are in no_main project
    #[cfg(test)]
    test_main();

    loop {}
}

/// This is the panic handler
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rudos::test_panic_handler(info)
}

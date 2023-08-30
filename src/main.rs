#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rudos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use rudos::println;

entry_point!(kernel_main);

/// This is the typechecked entry point of our system
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rudos::memory;
    use x86_64::{structures::paging::Translate, VirtAddr};

    println!("Hello World{}", "!");
    rudos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };
    let addresses = [
        0xb8000,
        0x201008,
        0x0100_0020_1a10,
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

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

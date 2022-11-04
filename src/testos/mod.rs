#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod tests;
pub mod vga_buffer;

pub fn init() {
	gdt::init();
	interrupts::init_idt();
	unsafe { interrupts::PICS.lock().initialize() };
	x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
	loop {
		x86_64::instructions::hlt();
	}
}

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
	init();
	test_main();
	hlt_loop();
}
#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
	tests::test_panic_handler(info)
}

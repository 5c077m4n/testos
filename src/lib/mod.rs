#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod qemu;
pub mod serial;
pub mod vga_buffer;
pub mod interrupts;

use core::{any::type_name, panic::PanicInfo, prelude::rust_2021::*};

pub trait Testable {
	fn run(&self) -> ();
}

impl<T> Testable for T
where
	T: Fn(),
{
    fn run(&self) {
		serial_print!("{}...\t", type_name::<T>());
		self();
		serial_println!("[ok]");
	}
}

pub fn test_runner(tests: &[&dyn Testable]) {
	serial_println!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}
	qemu::exit(qemu::ExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
	serial_println!("[failed]\n");
	serial_println!("Error: {}\n", info);
	qemu::exit(qemu::ExitCode::Fail);
	loop {}
}

#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
	test_main();
	loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	test_panic_handler(info)
}

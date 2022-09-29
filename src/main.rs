#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use testos::*;
use x86_64::VirtAddr;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	hlt_loop();
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
	println!("HELLO FROM TESTOS");
	init();

	let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
	let level_4_table = unsafe { memory::active_level_4_table(phys_mem_offset) };

	for (i, entry) in level_4_table.iter().enumerate() {
		if !entry.is_unused() {
			println!("L4 Entry {}: {:?}", i, entry);
		}
	}

	#[cfg(test)]
	test_main();

	hlt_loop();
}
entry_point!(kernel_main);

#[cfg(test)]
fn test_runner(tests: &[&dyn tests::Testable]) {
	serial_println!("Running {} tests", tests.len());
	for test in tests {
		test.run();
	}

	qemu::exit(qemu::ExitCode::Success);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	serial_println!("[failed]\n");
	serial_println!("Error: {}\n", info);
	qemu::exit(qemu::ExitCode::Fail);

	loop {}
}

#[test_case]
fn sanity() {
	assert_eq!(1, 1);
}

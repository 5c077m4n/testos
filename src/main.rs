#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use testos::*;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	println!("{}", info);
	hlt_loop();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("HELLO FROM TESTOS");
	init();

	#[cfg(test)]
	test_main();

	hlt_loop();
}

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

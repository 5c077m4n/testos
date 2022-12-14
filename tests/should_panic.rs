#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(testos::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use testos::{qemu, serial_print, serial_println};

#[no_mangle]
pub extern "C" fn _start() -> ! {
	should_fail();

	serial_println!("[test did not panic]");
	qemu::exit(qemu::ExitCode::Fail);
	loop {}
}

fn should_fail() {
	serial_print!("should_panic::should_fail...\t");
	assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
	serial_println!("[ok]");
	qemu::exit(qemu::ExitCode::Success);

	loop {}
}

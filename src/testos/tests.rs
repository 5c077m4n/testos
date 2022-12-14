use core::{any::type_name, panic::PanicInfo, prelude::rust_2021::*};

use super::{hlt_loop, qemu, serial_print, serial_println};

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

	hlt_loop();
}

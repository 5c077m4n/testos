#![allow(dead_code)]

use core::{
	cmp::{Eq, PartialEq},
	prelude::rust_2021::*,
};

const SERIAL_PORT: u16 = 0xf4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
	Success = 0x10,
	Fail = 0x11,
}

pub fn exit(exit_code: ExitCode) {
	use x86_64::instructions::port::Port;

	unsafe {
		let mut port = Port::new(SERIAL_PORT);
		port.write(exit_code as u32);
	}
}

use x86_64::instructions::port::Port;

static mut SERIAL_PORT: Port<u32> = Port::new(0xf4);

pub enum ExitCode {
	Success,
	Fail,
}
impl ExitCode {
	pub fn as_u32(self) -> u32 {
		match self {
			Self::Success => 0x10,
			Self::Fail => 0x11,
		}
	}
}

pub fn exit(exit_code: ExitCode) {
	unsafe {
		SERIAL_PORT.write(exit_code.as_u32());
	}
}

#![allow(dead_code)]

use core::{
	fmt::{Arguments, Result, Write},
	ops::{Deref, DerefMut},
	prelude::rust_2021::*,
};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_BUFFER: usize = 0xb8000;

lazy_static! {
	pub static ref WRITER: Mutex<Writer<'static>> = Mutex::new(Writer::default());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Color {
	#[default]
	Black = 0,
	Blue = 1,
	Green = 2,
	Cyan = 3,
	Red = 4,
	Magenta = 5,
	Brown = 6,
	LightGray = 7,
	DarkGray = 8,
	LightBlue = 9,
	LightGreen = 10,
	LightCyan = 11,
	LightRed = 12,
	Pink = 13,
	Yellow = 14,
	White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
	fn new(foreground: Color, background: Color) -> Self {
		Self((background as u8) << 4 | (foreground as u8))
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
	ascii_character: u8,
	color_code: ColorCode,
}
impl Deref for ScreenChar {
	type Target = Self;
	fn deref(&self) -> &Self::Target {
		&self
	}
}
impl DerefMut for ScreenChar {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self
	}
}

#[repr(transparent)]
struct Buffer {
	chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer<'w> {
	column_position: usize,
	color_code: ColorCode,
	buffer: &'w mut Buffer,
}
impl<'w> Default for Writer<'w> {
	fn default() -> Self {
		Self {
			column_position: 0,
			color_code: ColorCode::new(Color::Yellow, Color::default()),
			buffer: unsafe { &mut *(VGA_BUFFER as *mut Buffer) },
		}
	}
}
impl<'w> Writer<'w> {
	fn clear_row(&mut self, row: usize) {
		let blank = ScreenChar {
			ascii_character: b' ',
			color_code: self.color_code,
		};
		for col in 0..BUFFER_WIDTH {
			self.buffer.chars[row][col].write(blank);
		}
	}
	fn new_line(&mut self) {
		for row in 1..BUFFER_HEIGHT {
			for col in 0..BUFFER_WIDTH {
				let character = self.buffer.chars[row][col].read();
				self.buffer.chars[row - 1][col].write(character);
			}
		}
		self.clear_row(BUFFER_HEIGHT - 1);
		self.column_position = 0;
	}
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.new_line(),
			byte => {
				if self.column_position >= BUFFER_WIDTH {
					self.new_line();
				}

				let row = BUFFER_HEIGHT - 1;
				let col = self.column_position;
				let color_code = self.color_code;

				self.buffer.chars[row][col] = Volatile::new(ScreenChar {
					ascii_character: byte,
					color_code,
				});
				self.column_position += 1;
			}
		}
	}
	pub fn write(&mut self, s: &str) {
		for byte in s.bytes() {
			match byte {
				// printable ASCII byte or newline
				b' '..=b'~' | b'\n' => self.write_byte(byte),
				// not part of printable ASCII range
				_ => self.write_byte(0xfe),
			}
		}
	}
}
impl<'w> Write for Writer<'w> {
	fn write_str(&mut self, s: &str) -> Result {
		self.write(s);
		Result::Ok(())
	}
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
	interrupts::without_interrupts(|| {
		WRITER.lock().write_fmt(args).unwrap();
	});
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", core::format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test_case]
	fn test_println_simple() {
		println!("test_println_simple output");
	}
	#[test_case]
	fn test_println_many() {
		for _ in 0..200 {
			println!("test_println_many output");
		}
	}
	#[test_case]
	fn test_println_output() {
		let s = "Some test string that fits on a single line";
		println!("{}", s);

		for (i, c) in s.chars().enumerate() {
			let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
			assert_eq!(char::from(screen_char.ascii_character), c);
		}
	}
}

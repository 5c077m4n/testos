use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::{
	instructions::port::Port,
	registers::control::Cr2,
	structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use super::{gdt, hlt_loop, print, println};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
	Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
static mut KEYBOARD_DATA_PORT: Port<u8> = Port::new(0x60);

#[derive(Debug, Clone, Copy)]
pub enum InterruptIndex {
	Timer,
	Keyboard,
}
impl InterruptIndex {
	fn as_u8(self) -> u8 {
		match self {
			Self::Timer => PIC_1_OFFSET,
			Self::Keyboard => 33u8,
		}
	}
	fn as_usize(self) -> usize {
		self.as_u8() as usize
	}
}

lazy_static! {
	static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
		Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
	);
	static ref IDT: InterruptDescriptorTable = {
		let mut idt = InterruptDescriptorTable::new();
		idt.breakpoint.set_handler_fn(breakpoint_handler);
		idt.page_fault.set_handler_fn(page_fault_handler);
		idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
		idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

		unsafe {
			idt.double_fault
				.set_handler_fn(double_fault_handler)
				.set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
		}
		idt
	};
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
	println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn double_fault_handler(
	stack_frame: InterruptStackFrame,
	_error_code: u64,
) -> ! {
	panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
	print!(".");
	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
	};
}
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
	let mut keyboard = KEYBOARD.lock();
	let scancode: u8 = unsafe { KEYBOARD_DATA_PORT.read() };

	if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
		if let Some(key) = keyboard.process_keyevent(key_event) {
			match key {
				DecodedKey::Unicode(character) => print!("{}", character),
				DecodedKey::RawKey(key) => print!("{:?}", key),
			}
		}
	}

	unsafe {
		PICS.lock()
			.notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
	}
}
extern "x86-interrupt" fn page_fault_handler(
	stack_frame: InterruptStackFrame,
	error_code: PageFaultErrorCode,
) {
	println!("EXCEPTION: PAGE FAULT");
	println!("Accessed Address: {:?}", Cr2::read());
	println!("Error Code: {:?}", error_code);
	println!("{:#?}", stack_frame);
	hlt_loop();
}

pub fn init_idt() {
	IDT.load();
}

#[test_case]
fn test_breakpoint_exception() {
	use x86_64::instructions::interrupts;

	interrupts::int3();
}

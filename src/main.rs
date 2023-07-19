#![no_std]
#![no_main]
#![feature(lang_items)]

static mut PINB: *mut u8 = 0x23 as *mut u8;
static mut DDRB: *mut u8 = 0x24 as *mut u8;
static mut PORTB: *mut u8 = 0x25 as *mut u8;

static mut PIND: *mut u8 = 0x29 as *mut u8;
static mut DDRD: *mut u8 = 0x2A as *mut u8;
static mut PORTD: *mut u8 = 0x2B as *mut u8;

#[no_mangle]
fn main() -> ! {
    unsafe {
	*DDRB |= 1 << 0;
	*DDRD |= 1 << 5;
    }

    unsafe { helper_usb_init(); }
    while !is_initialized() {
	unsafe {
	    *PORTB ^= 1 << 0;
	    *PORTD ^= 1 << 5;
	}
	delay_ms(100);
    }

    loop {
	unsafe {
	    usb_send();
	}
    }
}

fn is_initialized() -> bool {
    unsafe { usb_state == UsbState::Attached }
}

fn delay_ms(ms: u8) {
    const clock_speed_ms: u32 = 16_000;
    for _ in 0..clock_speed_ms {
	for _ in 0..ms {
	    unsafe { core::arch::asm!("nop"); }
	}
    }
}

// repr(C) generates integers which are too large??
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u16)]
enum UsbState {
    Unknown,
    Disconnected,
    Attached,
}

#[link(name = "usb")]
extern "C" {
    fn helper_usb_init();
    fn usb_send();
    static usb_state: UsbState;
    static keyboard_pressed_keys: [u8; 6];
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn eh_personality() { }

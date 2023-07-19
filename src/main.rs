#![feature(asm_experimental_arch)]
#![feature(lang_items)]
#![no_main]
#![no_std]

static mut PINB: *mut u8 = 0x23 as *mut u8;
static mut DDRB: *mut u8 = 0x24 as *mut u8;
static mut PORTB: *mut u8 = 0x25 as *mut u8;

static mut PIND: *mut u8 = 0x29 as *mut u8;
static mut DDRD: *mut u8 = 0x2A as *mut u8;
static mut PORTD: *mut u8 = 0x2B as *mut u8;

#[no_mangle]
fn main() -> ! {
    unsafe {
	// set LEDs to out
        *DDRB |= 1 << 0;
        *DDRD |= 1 << 5;

	// set buttons to in
	*DDRD &= !(1 << 1 | 1 << 0 | 1 << 4);

	// set buttons to high, so they can be pulled low
	*PORTD |= 1 << 1 | 1 << 0 | 1 << 4;
    }

    unsafe {
        helper_usb_init();
    }
    while !is_initialized() {
        unsafe {
            *PORTB ^= 1 << 0;
            *PORTD ^= 1 << 5;
        }
        delay_ms(100);
    }

    unsafe {
	*PORTB |= 1 << 0;
	*PORTD |= 1 << 5;
    }

    let ports: &[(u8, u8)] = &[(1, 0x04), (0, 0x05), (4, 0x06)];
    loop {
	for (i, (offset, scancode)) in ports.iter().enumerate() {
	    unsafe {
		if *PIND & (1 << offset) == 0 {
		    keyboard_pressed_keys[i] = *scancode;
		} else {
		    keyboard_pressed_keys[i] = 0;
		}
	    }
	}
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
            unsafe {
                core::arch::asm!("nop");
            }
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
    static mut keyboard_pressed_keys: [u8; 6];
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn eh_personality() {}

#![allow(dead_code)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(asm_experimental_arch)]
#![feature(lang_items)]

const PINB: *mut u8 = 0x23 as *mut u8;
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

const PIND: *mut u8 = 0x29 as *mut u8;
const DDRD: *mut u8 = 0x2A as *mut u8;
const PORTD: *mut u8 = 0x2B as *mut u8;

#[cfg(not(test))]
#[panic_handler]
pub extern "C" fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    main_impl();
}

fn main_impl() -> ! {
    unsafe {
	*DDRB |= 1;
	*DDRD |= 1 << 5;

	*PORTB |= 1;
    }
    loop {
	unsafe {
	    *PORTB ^= 1;
	    *PORTD ^= 1 << 5;
	}
	delay_ms(1000);
    }
}

fn delay_ms(milliseconds: usize) {
    const CYCLES_PER_MILLISECOND: usize = 1600;
    for _ in 0..milliseconds {
	for _ in 0..CYCLES_PER_MILLISECOND {
	    unsafe { core::arch::asm!("nop") }
	}
    }
}

#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    loop {}
}

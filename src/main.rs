#![feature(asm_experimental_arch)]
#![feature(lang_items)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

mod usb;

#[cfg(not(test))]
#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);

    let mut rx_led = pins.led_rx.into_output();
    let mut tx_led = pins.led_tx.into_output();

    unsafe {
	let contents = usb::DEVICE_DESCRIPTOR.serialize();
	core::mem::forget(contents);
	helper_usb_init(contents.as_ptr());
    }
    while !is_initialized() {
	rx_led.toggle();
	tx_led.toggle();
	arduino_hal::delay_ms(100);
    }
    rx_led.set_high();
    tx_led.set_high();

    let pins = [
	(pins.d2.into_pull_up_input().downgrade(), 0x04),
	(pins.d3.into_pull_up_input().downgrade(), 0x05),
	(pins.d4.into_pull_up_input().downgrade(), 0x06),
    ];
    loop {
	for (i, (pin, scancode)) in pins.iter().enumerate() {
	    let value;
	    if pin.is_low() {
		value = *scancode;
	    } else {
		value = 0;
	    }
	    unsafe {
		keyboard_pressed_keys[i] = value;
	    }
	}
	unsafe { usb_send(); }
    }
}

#[cfg(not(test))]
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

#[cfg(not(test))]
#[link(name = "usb")]
extern "C" {
    fn helper_usb_init(device_descriptor: *const u8);
    fn usb_send();
    static usb_state: UsbState;
    static mut keyboard_pressed_keys: [u8; 6];
}

#[cfg(not(test))]
#[link(name = "c")]
extern "C" {}

#[cfg(not(test))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn abort() -> ! { loop {} }

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn eh_personality() {}

#![feature(lang_items)]
#![no_main]
#![no_std]

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.led_rx.into_output();

    led.set_low();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

// TODO: why is this here? why is it not provided by anything else?
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! { loop {} }

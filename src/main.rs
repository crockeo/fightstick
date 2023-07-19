#![no_std]
#![no_main]
#![feature(lang_items)]

#[no_mangle]
fn main() -> ! {
    loop {}
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn eh_personality() {
}

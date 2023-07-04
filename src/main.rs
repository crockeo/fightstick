#![feature(lang_items)]
#![no_main]
#![no_std]

#[no_mangle]
pub extern "C" fn main() {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Several libraries we include generate code that depends on libc
// but we don't have libc available on our target microcontroller.
// We provide a small subset of libc here to satisfy those dependencies.
use core::ffi::c_int;
use core::ffi::c_size_t;

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: c_size_t) {
    for i in 0..n {
        *dst.add(i) = *src.add(i);
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, c: c_int, n: c_size_t) {
    for i in 0..n {
        *dst.add(i) = c as u8;
    }
}

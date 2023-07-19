fn main() {
    println!("cargo:rustc-link-search=/opt/homebrew/lib/avr-gcc/9");
    println!("cargo:rustc-link-lib=static=c");
}

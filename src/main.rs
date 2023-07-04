#![feature(abi_avr_interrupt)]
#![feature(c_size_t)]
#![feature(lang_items)]
#![no_main]
#![no_std]

use core::ffi::c_int;
use core::ffi::c_size_t;

use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::Output;
use arduino_hal::port::mode::PullUp;
use arduino_hal::port::Pin;
use atmega_usbd::UsbBus;
use avr_device::interrupt;
use panic_halt as _;
use usb_device::bus::UsbBusAllocator;
use usb_device::device::UsbDevice;
use usb_device::device::UsbDeviceBuilder;
use usb_device::device::UsbVidPid;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;

mod libc;

const PAYLOAD: &[u8] = b"Hello World";
static mut USB_CTX: Option<UsbContext> = None;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);

    let status = pins.led_tx.into_output();
    let trigger = pins.d2.into_pull_up_input();

    // Magical incantation which does some nonsense to something called PLL.
    // TODO: figure out what this is.
    let pll = peripherals.PLL;
    pll.pllcsr.write(|w| w.pindiv().set_bit());
    pll.pllfrq
        .write(|w| w.pdiv().mhz96().plltm().factor_15().pllusb().set_bit());
    pll.pllcsr.modify(|_, w| w.plle().set_bit());
    while pll.pllcsr.read().plock().bit_is_clear() {}

    let usb_bus = unsafe {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;
        USB_BUS.insert(UsbBus::new(peripherals.USB_DEVICE))
    };
    let hid_class = HIDClass::new(usb_bus, KeyboardReport::desc(), 60);
    let usb_device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0x0001))
        .manufacturer("Foo")
        .product("Bar")
        .build();

    unsafe {
        USB_CTX = Some(UsbContext {
            hid_class: hid_class,
            usb_device: usb_device,
            current_index: 0,
            indicator: status.downgrade(),
            trigger: trigger.downgrade(),
        });
    }

    unsafe {
        interrupt::enable();
    }

    loop {
        avr_device::asm::sleep();
    }
}

#[interrupt(atmega32u4)]
fn USB_GEN() {
    unsafe {
        poll_usb();
    }
}

#[interrupt(atmega32u4)]
fn USB_COM() {
    unsafe {
        poll_usb();
    }
}

unsafe fn poll_usb() {
    let ctx = USB_CTX.as_mut().unwrap();
    ctx.poll();
}

struct UsbContext {
    usb_device: UsbDevice<'static, UsbBus>,
    hid_class: HIDClass<'static, UsbBus>,
    current_index: usize,
    indicator: Pin<Output>,
    trigger: Pin<Input<PullUp>>,
}

impl UsbContext {
    fn poll(&mut self) {
        if self.trigger.is_low() {
            self.indicator.set_high();

            if let Some(report) = PAYLOAD
                .get(self.current_index)
                .copied()
                .and_then(ascii_to_report)
            {
                if self.hid_class.push_input(&report).is_ok() {
                    self.current_index += 1;
                }
            } else {
                self.hid_class.push_input(&BLANK_REPORT).ok();
            }
        } else {
            self.indicator.set_low();

            self.current_index = 0;
            self.hid_class.push_input(&BLANK_REPORT).ok();
        }

        if self.usb_device.poll(&mut [&mut self.hid_class]) {
            let mut report_buf = [0u8; 1];

            if self.hid_class.pull_raw_output(&mut report_buf).is_ok() {
                if report_buf[0] & 2 != 0 {
                    self.indicator.set_high();
                } else {
                    self.indicator.set_low();
                }
            }
        }
    }
}

const BLANK_REPORT: KeyboardReport = KeyboardReport {
    modifier: 0,
    reserved: 0,
    leds: 0,
    keycodes: [0; 6],
};

fn ascii_to_report(c: u8) -> Option<KeyboardReport> {
    let (keycode, shift) = if c.is_ascii_alphabetic() {
        (c.to_ascii_lowercase() - b'a' + 0x04, c.is_ascii_uppercase())
    } else {
        match c {
            b' ' => (0x2c, false),
            _ => return None,
        }
    };

    let mut report = BLANK_REPORT;
    if shift {
        report.modifier |= 0x2;
    }
    report.keycodes[0] = keycode;
    Some(report)
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

// TODO: why is this here? why is it not provided by anything else?
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    loop {}
}

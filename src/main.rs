#![feature(abi_avr_interrupt)]
#![feature(c_size_t)]
#![feature(lang_items)]
#![no_main]
#![no_std]

use arduino_hal::pac::USART1;
use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::usart::Usart;
use atmega_hal::port::PD2;
use atmega_hal::port::PD3;
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

mod inputs;
mod libc;

const PAYLOAD: &[u8] = b"Hello World";
static mut USB_CTX: Option<UsbContext> = None;

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(peripherals);

    let mut indicator = pins.led_tx.into_output();
    indicator.set_high();
    pins.led_rx.into_output().set_high();

    let serial = arduino_hal::default_serial!(peripherals, pins, 115200);

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
            hid_class,
            usb_device,
            serial,
            indicator: indicator.downgrade(),
            input_reader: inputs::InputReader::new([
                // start / select
                pins.a2.into_pull_up_input().downgrade(),
                pins.a3.into_pull_up_input().downgrade(),
                // up / down / left / right
                pins.d2.into_pull_up_input().downgrade(),
                pins.d3.into_pull_up_input().downgrade(),
                pins.d4.into_pull_up_input().downgrade(),
                pins.d5.into_pull_up_input().downgrade(),
                // a / b / x / y
                pins.d6.into_pull_up_input().downgrade(),
                pins.d7.into_pull_up_input().downgrade(),
                pins.d8.into_pull_up_input().downgrade(),
                pins.d9.into_pull_up_input().downgrade(),
                // r1 / r2 / r3
                pins.d10.into_pull_up_input().downgrade(),
                pins.d16.into_pull_up_input().downgrade(),
                pins.d14.into_pull_up_input().downgrade(),
                // l1 / l2 / l3
                pins.d15.into_pull_up_input().downgrade(),
                pins.a0.into_pull_up_input().downgrade(),
                pins.a1.into_pull_up_input().downgrade(),
            ]),
            report_queue: ReportQueue::new(),
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
    serial: Usart<USART1, Pin<Input, PD2>, Pin<Output, PD3>>,
    indicator: Pin<Output>,
    input_reader: inputs::InputReader,
    report_queue: ReportQueue,
}

impl UsbContext {
    fn poll(&mut self) {
        if self.report_queue.empty() {
            self.indicator.set_high();
            let input_map = self.input_reader.read();
            let (reports, num_populated) = input_map.into_keyboard_reports();
            self.report_queue.replace(reports, num_populated);
        }

        if let Some(report) = self.report_queue.pop() {
            self.indicator.set_low();
            _ = self.hid_class.push_input(&report);
        }
    }
}

const QUEUE_SIZE: usize = 3;

struct ReportQueue {
    queue: [KeyboardReport; QUEUE_SIZE],
    head: usize,
}

impl ReportQueue {
    fn new() -> Self {
        Self {
            queue: [inputs::EMPTY_REPORT; QUEUE_SIZE],
            head: QUEUE_SIZE,
        }
    }

    fn replace(&mut self, reports: [KeyboardReport; QUEUE_SIZE], num_populated: usize) {
        let diff = QUEUE_SIZE - num_populated;
        for i in 0..num_populated {
            self.queue[i + diff] = reports[i];
        }
        self.head = diff;
    }

    fn pop(&mut self) -> Option<KeyboardReport> {
        if self.empty() {
            return None;
        }
        let report = self.queue[self.head];
        self.head += 1;
        Some(report)
    }

    fn empty(&self) -> bool {
        self.head >= self.queue.len()
    }
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

// TODO: why is this here? why is it not provided by anything else?
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    loop {}
}

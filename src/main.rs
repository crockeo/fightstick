#![feature(abi_avr_interrupt)]
#![feature(c_size_t)]
#![feature(lang_items)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

// TODO: see if we can make this appear as an XInput controller
// so we can use button input directly instead of mapping onto characters

// TODO: maybe change how the InputReader works so that it only buffers the last 6 inputs instead of repeating inputs.
// maybe here = depending on testing to determine how this works in practice

mod inputs;
mod libc;

use arduino_hal::pac::USART1;
use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::Output;
use arduino_hal::port::Pin;
use arduino_hal::usart::Usart;
use atmega_hal::port::PD2;
use atmega_hal::port::PD3;
use atmega_usbd::UsbBus;
use avr_device::interrupt;
use usb_device::bus::UsbBusAllocator;
use usb_device::device::UsbDevice;
use usb_device::device::UsbDeviceBuilder;
use usb_device::device::UsbVidPid;
use usbd_hid::descriptor::KeyboardReport;
use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::hid_class::HIDClass;

use inputs::Button;
use inputs::GenericControllerReport;
use inputs::InputMode;
use inputs::InputReader;
use inputs::EMPTY_REPORT;
use inputs::INPUT_MODE;

#[cfg(not(test))]
use panic_halt as _;

#[cfg(not(test))]
#[arduino_hal::entry]
fn avr_entry() -> ! {
    main()
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[interrupt(atmega32u4)]
fn USB_GEN() {
    unsafe {
        poll_usb();
    }
}

#[cfg(not(test))]
#[interrupt(atmega32u4)]
fn USB_COM() {
    unsafe {
        poll_usb();
    }
}

static mut USB_CTX: Option<UsbContext> = None;

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

    let hid_class = match INPUT_MODE {
        InputMode::Controller => HIDClass::new(usb_bus, GenericControllerReport::desc(), 60),
        InputMode::Keyboard => HIDClass::new(usb_bus, KeyboardReport::desc(), 60),
    };
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
            input_reader: InputReader::new([
                (pins.a2.into_pull_up_input().downgrade(), Button::Start),
                (pins.a3.into_pull_up_input().downgrade(), Button::Select),
                (pins.d2.into_pull_up_input().downgrade(), Button::Up),
                (pins.d3.into_pull_up_input().downgrade(), Button::Down),
                (pins.d4.into_pull_up_input().downgrade(), Button::Left),
                (pins.d5.into_pull_up_input().downgrade(), Button::Right),
                (pins.d8.into_pull_up_input().downgrade(), Button::LightPunch),
                (
                    pins.d9.into_pull_up_input().downgrade(),
                    Button::MediumPunch,
                ),
                (
                    pins.d10.into_pull_up_input().downgrade(),
                    Button::HeavyPunch,
                ),
                (pins.d6.into_pull_up_input().downgrade(), Button::LightKick),
                (pins.d7.into_pull_up_input().downgrade(), Button::MediumKick),
                (pins.d16.into_pull_up_input().downgrade(), Button::HeavyKick),
                (pins.d15.into_pull_up_input().downgrade(), Button::Macro1),
                (pins.a0.into_pull_up_input().downgrade(), Button::Macro2),
                (pins.d14.into_pull_up_input().downgrade(), Button::Macro3),
                (pins.a1.into_pull_up_input().downgrade(), Button::Macro4),
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

unsafe fn poll_usb() {
    let ctx = USB_CTX.as_mut().unwrap();
    ctx.poll();
}

struct UsbContext {
    usb_device: UsbDevice<'static, UsbBus>,
    hid_class: HIDClass<'static, UsbBus>,
    serial: Usart<USART1, Pin<Input, PD2>, Pin<Output, PD3>>,
    indicator: Pin<Output>,
    input_reader: InputReader,
    report_queue: ReportQueue,
}

impl UsbContext {
    fn poll(&mut self) {
        if !self.usb_device.poll(&mut [&mut self.hid_class]) {
            return;
        }

        match INPUT_MODE {
            InputMode::Controller => self.poll_controller(),
            InputMode::Keyboard => self.poll_keyboard(),
        }

    }

    fn poll_controller(&mut self) {
        let input = self.input_reader.read();
        if input.empty() {
            self.indicator.set_high();
        } else {
            self.indicator.set_low();
        }

        let report = input.into_controller_report();
        let _ = self.hid_class.push_input(&report);
    }

    fn poll_keyboard(&mut self) {
        if self.report_queue.empty() {
            self.indicator.set_high();
            let input_map = self.input_reader.read();
            let (reports, num_populated) = input_map.into_keyboard_reports();
            self.report_queue.replace(reports, num_populated);
        }

        if let Some(report) = self.report_queue.pop() {
            self.indicator.set_low();
            let _ = self.hid_class.push_input(&report);
        } else {
            let _ = self.hid_class.push_input(&EMPTY_REPORT);
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
            queue: [EMPTY_REPORT; QUEUE_SIZE],
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

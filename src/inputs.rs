use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::PullUp;
use arduino_hal::port::Pin;
use usbd_hid::descriptor::KeyboardReport;

pub struct InputReader {
    pins: [Pin<Input<PullUp>>; 16],
}

impl InputReader {
    pub fn new(pins: [Pin<Input<PullUp>>; 16]) -> Self {
        // TODO: maybe make these map onto real, specific pins
        // so we don't have the runtime overhead?
        Self { pins }
    }

    pub fn read(&self) -> InputMap {
        let mut bitmap: u16 = 0;
        for (i, pin) in self.pins.iter().enumerate() {
            if pin.is_low() {
                bitmap |= 1 << i;
            }
        }
        InputMap(bitmap)
    }
}

pub struct InputMap(u16);

impl InputMap {
    pub fn into_keyboard_reports(self) -> ([KeyboardReport; 3], usize) {
        let keycodes = [
            0x29, // start -> escape
            0x32, // select -> `
            0x1a, // up -> w
            0x16, // down -> s
            0x04, // left -> a
            0x07, // right -> d
            0x1e, // a -> j
            0x1f, // b -> k
            0x18, // x -> u
            0x1b, // y -> i
            0x12, // r1 -> o
            0x0f, // r2 -> l
            0x13, // r3 -> 1
            0x13, // l1 -> p
            0x33, // l2 -> ;
            0x1f, // l3 -> 2
        ];

        let mut reports = [EMPTY_REPORT; 3];
        for (i, scancode) in keycodes.into_iter().enumerate() {
            let report = &mut reports[i / 6];
            let pressed = self.0 & (1 << (15 - i)) != 0;
            if pressed {
                report.keycodes[i % 6] = scancode;
            }
        }
        (reports, 3)
    }
}

pub const EMPTY_REPORT: KeyboardReport = KeyboardReport {
    modifier: 0,
    reserved: 0,
    leds: 0,
    keycodes: [0; 6],
};

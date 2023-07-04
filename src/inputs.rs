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
            0x35, // select -> `
            scancode('w'), // up -> w
            scancode('s'), // down -> s
            scancode('a'), // left -> a
            scancode('d'), // right -> d
            scancode('j'), // a -> j
            scancode('k'), // b -> k
            scancode('u'), // x -> u
            scancode('i'), // y -> i
            scancode('o'), // r1 -> o
            scancode('l'), // r2 -> l
            0x1e, // r3 -> 1
            scancode('p'), // l1 -> p
            0x33, // l2 -> ;
            0x1f, // l3 -> 2
        ];

        let mut reports = [EMPTY_REPORT; 3];
        for (i, scancode) in keycodes.into_iter().enumerate() {
            let report = &mut reports[i / 6];
            let pressed = self.0 & (1 << i) != 0;
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

fn scancode(c: char) -> u8 {
    (c as u8 - b'a') + 0x04
}

use core::slice::Iter;

use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::PullUp;
use arduino_hal::port::Pin;
use usbd_hid::descriptor::KeyboardReport;

pub const INPUT_MAP_WIDTH: usize = 16;

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum Button {
    Start       = 0b0000_0000_0000_0001,
    Select      = 0b0000_0000_0000_0010,
    Up          = 0b0000_0000_0000_0100,
    Down        = 0b0000_0000_0000_1000,
    Left        = 0b0000_0000_0001_0000,
    Right       = 0b0000_0000_0010_0000,
    LightPunch  = 0b0000_0000_0100_0000,
    MediumPunch = 0b0000_0000_1000_0000,
    HeavyPunch  = 0b0000_0001_0000_0000,
    LightKick   = 0b0000_0010_0000_0000,
    MediumKick  = 0b0000_0100_0000_0000,
    HeavyKick   = 0b0000_1000_0000_0000,
    Macro1      = 0b0001_0000_0000_0000,
    Macro2      = 0b0010_0000_0000_0000,
    Macro3      = 0b0100_0000_0000_0000,
    Macro4      = 0b1000_0000_0000_0000,
}

impl Button {
    fn iterator() -> Iter<'static, Button> {
        static BUTTONS: [Button; INPUT_MAP_WIDTH] = [
            Button::Start,
            Button::Select,
            Button::Up,
            Button::Down,
            Button::Left,
            Button::Right,
            Button::LightPunch,
            Button::MediumPunch,
            Button::HeavyPunch,
            Button::LightKick,
            Button::MediumKick,
            Button::HeavyKick,
            Button::Macro1,
            Button::Macro2,
            Button::Macro3,
            Button::Macro4,
        ];
        BUTTONS.iter()
    }

    const fn keyboard_scancode(self) -> u8 {
        match self {
            Button::Start => 0x29,                              // start -> escape
            Button::Select => 0x35,                             // select -> `
            Button::Up => keyboard_char_scancode('w'),          // up -> w
            Button::Down => keyboard_char_scancode('s'),        // down -> s
            Button::Left => keyboard_char_scancode('a'),        // left -> a
            Button::Right => keyboard_char_scancode('d'),       // right -> d
            Button::LightPunch => keyboard_char_scancode('u'),  // x -> u
            Button::MediumPunch => keyboard_char_scancode('i'), // y -> i
            Button::HeavyPunch => keyboard_char_scancode('o'),  // r1 -> o
            Button::LightKick => keyboard_char_scancode('j'),   // a -> j
            Button::MediumKick => keyboard_char_scancode('k'),  // b -> k
            Button::HeavyKick => keyboard_char_scancode('l'),   // r2 -> l
            Button::Macro1 => keyboard_char_scancode('p'),      // l1 -> p
            Button::Macro2 => 0x33,                             // l2 -> ;
            Button::Macro3 => 0x1e,                             // r3 -> 1
            Button::Macro4 => 0x1f,                             // l3 -> 2
        }
    }
}

pub struct InputReader {
    pins: [(Pin<Input<PullUp>>, Button); INPUT_MAP_WIDTH],
}

impl InputReader {
    pub fn new(pins: [(Pin<Input<PullUp>>, Button); INPUT_MAP_WIDTH]) -> Self {
        // TODO: maybe make these map onto real, specific pins
        // so we don't have the runtime overhead?
        Self { pins }
    }

    pub fn read(&self) -> InputMap {
        let mut bitmap: u16 = 0;
        for (pin, button) in self.pins.iter() {
            if pin.is_low() {
                bitmap |= *button as u16;
            }
        }
        InputMap(bitmap)
    }
}

pub struct InputMap(u16);

impl InputMap {
    pub fn into_keyboard_reports(self) -> ([KeyboardReport; 3], usize) {
        let mut reports = [EMPTY_REPORT; 3];
        let mut report_index = 0;
        for button in Button::iterator() {
            let report = &mut reports[report_index / 6];
            let pressed = self.0 & *button as u16 != 0;
            if pressed {
                report.keycodes[report_index % 6] = button.keyboard_scancode();
                report_index += 1;
            }
        }

        let mut len = report_index / 6;
        if report_index % 6 != 0 {
            len += 1;
        }
        (reports, len)
    }
}

const fn keyboard_char_scancode(c: char) -> u8 {
    (c as u8 - b'a') + 0x04
}

pub const EMPTY_REPORT: KeyboardReport = KeyboardReport {
    modifier: 0,
    reserved: 0,
    leds: 0,
    keycodes: [0; 6],
};

#[cfg(test)]
mod tests {
    use super::*;

    const KEYCODES: [u8; INPUT_MAP_WIDTH] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    #[test]
    fn test_into_keyboard_reports_empty() {
        let (_, count) = InputMap(0).into_keyboard_reports(KEYCODES);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_into_keyboard_reports_single() {
        let (reports, count) = InputMap(0b1).into_keyboard_reports(KEYCODES);
        assert_eq!(count, 1);
        assert_eq!(reports[0].keycodes[0], 1);
    }

    #[test]
    fn test_into_keyboard_reports_many() {
        let (reports, count) = InputMap(0b0011_1111).into_keyboard_reports(KEYCODES);
        assert_eq!(count, 1);
        assert_eq!(reports[0].keycodes, [1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_into_keyboard_reports_multipart() {
        let (reports, count) = InputMap(0b0011_1111).into_keyboard_reports(KEYCODES);
        assert_eq!(count, 1);
        assert_eq!(reports[0].keycodes, [1, 2, 3, 4, 5, 6]);
    }
}

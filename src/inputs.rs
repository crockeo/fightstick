use core::slice::Iter;

use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::PullUp;
use arduino_hal::port::Pin;
use usbd_hid::descriptor::gen_hid_descriptor;
use usbd_hid::descriptor::generator_prelude::*;
use usbd_hid::descriptor::KeyboardReport;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InputMode {
    Controller,
    Keyboard,
}

pub const INPUT_MODE: InputMode = InputMode::Controller;

#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = JOYSTICK) = {
	(usage_page = BUTTON, usage_min = 1, usage_max = 8) = {
	    #[packed_bits 8] #[item_settings data,variable,absolute] buttons0=input;
	};
	(usage_page = BUTTON, usage_min = 9, usage_max = 16) = {
	    #[packed_bits 8] #[item_settings data,variable,absolute] buttons1=input;
	};
    }
)]
pub struct GenericControllerReport {
    pub buttons0: u8,
    pub buttons1: u8,
}

pub const INPUT_MAP_WIDTH: usize = 16;

#[derive(Copy, Clone)]
#[repr(u16)]
pub enum Button {
    Start = 0b0000_0000_0000_0001,
    Select = 0b0000_0000_0000_0010,
    Up = 0b0000_0000_0000_0100,
    Down = 0b0000_0000_0000_1000,
    Left = 0b0000_0000_0001_0000,
    Right = 0b0000_0000_0010_0000,
    LightPunch = 0b0000_0000_0100_0000,
    MediumPunch = 0b0000_0000_1000_0000,
    HeavyPunch = 0b0000_0001_0000_0000,
    LightKick = 0b0000_0010_0000_0000,
    MediumKick = 0b0000_0100_0000_0000,
    HeavyKick = 0b0000_1000_0000_0000,
    Macro1 = 0b0001_0000_0000_0000,
    Macro2 = 0b0010_0000_0000_0000,
    Macro3 = 0b0100_0000_0000_0000,
    Macro4 = 0b1000_0000_0000_0000,
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
            Button::Start => 0x29,  // escape
            Button::Select => 0x35, // grave
            Button::Up => keyboard_char_scancode('w'),
            Button::Down => keyboard_char_scancode('s'),
            Button::Left => keyboard_char_scancode('a'),
            Button::Right => keyboard_char_scancode('d'),
            Button::LightPunch => keyboard_char_scancode('u'),
            Button::MediumPunch => keyboard_char_scancode('i'),
            Button::HeavyPunch => keyboard_char_scancode('o'),
            Button::LightKick => keyboard_char_scancode('j'),
            Button::MediumKick => keyboard_char_scancode('k'),
            Button::HeavyKick => keyboard_char_scancode('l'),
            Button::Macro1 => keyboard_char_scancode('p'),
            Button::Macro2 => 0x33, // ;
            Button::Macro3 => 0x1e, // 1
            Button::Macro4 => 0x1f, // 2
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
    pub fn empty(&self) -> bool {
        self.0 == 0
    }

    pub fn into_controller_report(self) -> GenericControllerReport {
        let buttons = self.0;
        let buttons0 = ((buttons & 0b1111_1111_0000_0000) >> 8) as u8;
        let buttons1 = (buttons & 0b0000_0000_1111_1111) as u8;
        GenericControllerReport { buttons0, buttons1 }
    }

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
    use Button::*;

    #[test]
    fn test_into_keyboard_reports_empty() {
        let (_, count) = InputMap(0).into_keyboard_reports();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_into_keyboard_reports_single() {
        let (reports, count) = InputMap(Button::Start as u16).into_keyboard_reports();
        assert_eq!(count, 1);
        assert_eq!(reports[0].keycodes[0], Button::Start.keyboard_scancode());
    }

    #[test]
    fn test_into_keyboard_reports_many() {
        let (reports, count) = InputMap(
            Start as u16 | Select as u16 | Up as u16 | Down as u16 | Left as u16 | Right as u16,
        )
        .into_keyboard_reports();
        assert_eq!(count, 1);
        assert_eq!(
            reports[0].keycodes,
            [
                Start.keyboard_scancode(),
                Select.keyboard_scancode(),
                Up.keyboard_scancode(),
                Down.keyboard_scancode(),
                Left.keyboard_scancode(),
                Right.keyboard_scancode(),
            ]
        );
    }

    #[test]
    fn test_into_keyboard_reports_multipart() {
        let (reports, count) = InputMap(0b1111_1111).into_keyboard_reports();
        assert_eq!(count, 2);
        assert_eq!(
            reports[0].keycodes,
            [
                Start.keyboard_scancode(),
                Select.keyboard_scancode(),
                Up.keyboard_scancode(),
                Down.keyboard_scancode(),
                Left.keyboard_scancode(),
                Right.keyboard_scancode(),
            ]
        );
        assert_eq!(
            reports[1].keycodes,
            [
                LightPunch.keyboard_scancode(),
                MediumPunch.keyboard_scancode(),
                0,
                0,
                0,
                0,
            ]
        );
    }
}

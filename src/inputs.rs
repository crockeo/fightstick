use arduino_hal::port::mode::Input;
use arduino_hal::port::mode::PullUp;
use arduino_hal::port::Pin;
use usbd_hid::descriptor::KeyboardReport;

pub const INPUT_MAP_WIDTH: usize = 16;

pub struct InputReader {
    pins: [Pin<Input<PullUp>>; INPUT_MAP_WIDTH],
}

impl InputReader {
    pub fn new(pins: [Pin<Input<PullUp>>; INPUT_MAP_WIDTH]) -> Self {
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
    pub fn into_keyboard_reports(
        self,
        scancodes: [u8; INPUT_MAP_WIDTH],
    ) -> ([KeyboardReport; 3], usize) {
        let mut reports = [EMPTY_REPORT; 3];
        let mut report_index = 0;
        for (i, scancode) in scancodes.into_iter().enumerate() {
            let report = &mut reports[report_index / 6];
            let pressed = self.0 & (1 << i) != 0;
            if pressed {
                report.keycodes[report_index % 6] = scancode;
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

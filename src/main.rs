#![allow(dead_code)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]
#![feature(asm_experimental_arch)]
#![feature(lang_items)]

mod usb_sys;

const PINB: *mut u8 = 0x23 as *mut u8;
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

const PIND: *mut u8 = 0x29 as *mut u8;
const DDRD: *mut u8 = 0x2A as *mut u8;
const PORTD: *mut u8 = 0x2B as *mut u8;

#[cfg(not(test))]
#[panic_handler]
pub extern "C" fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> ! {
    main_impl();
}

const KEYBOARD_REPORT_DESCRIPTOR: &[u8] = &[
    0x05,
    0x01,  // Usage Page - Generic Desktop - HID Spec Appendix E E.6 - The
           // values for the HID tags are not clearly listed anywhere really, so
           // this table is very useful
    0x09,
    0x06,  // Usage - Keyboard
    0xA1,
    0x01,  // Collection - Application
    0x05,
    0x07,  // Usage Page - Key Codes
    0x19,
    0xE0,  // Usage Minimum - The bit that controls the 8 modifier characters
           // (ctrl, command, etc)
    0x29,
    0xE7,  // Usage Maximum - The end of the modifier bit (0xE7 - 0xE0 = 1 byte)
    0x15,
    0x00,  // Logical Minimum - These keys are either not pressed or pressed, 0
           // or 1
    0x25,
    0x01,  // Logical Maximum - Pressed state == 1
    0x75,
    0x01,  // Report Size - The size of the IN report to the host
    0x95,
    0x08,  // Report Count - The number of keys in the report
    0x81,
    0x02,  // Input - These are variable inputs
    0x95,
    0x01,  // Report Count - 1
    0x75,
    0x08,  // Report Size - 8
    0x81,
    0x01,  // This byte is reserved according to the spec
    0x95,
    0x05,  // Report Count - This is for the keyboard LEDs
    0x75,
    0x01,  // Report Size
    0x05,
    0x08,  // Usage Page for LEDs
    0x19,
    0x01,  // Usage minimum for LEDs
    0x29,
    0x05,  // Usage maximum for LEDs
    0x91,
    0x02,  // Output - This is for a host output to the keyboard for the status
           // of the LEDs
    0x95,
    0x01,  // Padding for the report so that it is at least 1 byte
    0x75,
    0x03,  // Padding
    0x91,
    0x01,  // Output - Constant for padding
    0x95,
    0x06,  // Report Count - For the keys
    0x75,
    0x08,  // Report Size - For the keys
    0x15,
    0x00,  // Logical Minimum
    0x25,
    0x65,  // Logical Maximum
    0x05,
    0x07,  // Usage Page - Key Codes
    0x19,
    0x00,  // Usage Minimum - 0
    0x29,
    0x65,  // Usage Maximum - 101
    0x81,
    0x00,  // Input - Data, Array
    0xC0,   // End collection
];

const KEYBOARD_DEVICE_DESCRIPTOR: usb_sys::DeviceDescriptor = usb_sys::DeviceDescriptor {
    length: core::mem::size_of::<usb_sys::DeviceDescriptor>() as u8,
    descriptor_type: 1,
    usb_version: 0x0200,
    device_class: 0,
    device_subclass: 0,
    device_protocol: 0,
    max_packet_size: 32,
    vendor_id: 0xfeed,
    product_id: 0x0001,
    device_version: 0x0100,
    manufacturer_string_index: 0,
    product_string_index: 0,
    serial_number_string_index: 0,
    num_configurations: 1,
};

const KEYBOARD_CONFIG_DESCRIPTOR: usb_sys::ConfigurationDescriptor = usb_sys::ConfigurationDescriptor{
    length: core::mem::size_of::<usb_sys::ConfigurationDescriptor>() as u8,
    descriptor_type: 2,
    total_length: (
	core::mem::size_of::<usb_sys::ConfigurationDescriptor>()
	+ core::mem::size_of::<usb_sys::InterfaceDescriptor>()
	+ core::mem::size_of::<usb_sys::EndpointDescriptor>()
	+ core::mem::size_of::<usb_sys::HIDDescriptor>()
    ) as u16,
    num_interfaces: 1,
    configuration_value: 1,
    configuration_string_index: 0,
    attributes: 0xC0,
    max_power: 50,
};

const KEYBOARD_CONFIG_DESCRIPTORS: [*const usb_sys::ConfigurationDescriptor; 1] = [
    &KEYBOARD_CONFIG_DESCRIPTOR,
];

const KEYBOARD_INTERFACE_DESCRIPTOR: usb_sys::InterfaceDescriptor = usb_sys::InterfaceDescriptor {
    length: core::mem::size_of::<usb_sys::InterfaceDescriptor>() as u8,
    descriptor_type: 4,
    interface_number: 0,
    alternate_setting: 0,
    num_endpoints: 1,
    interface_class: 0x03, // interface class for HIDdescriptor
    interface_subclass: 0x01,  // boot subclass, because this is a keyboard :^)
    interface_protocol: 0x01,  // protocol for keyboard
    interface_string_index: 0,
};

const KEYBOARD_INTERFACE_DESCRIPTORS: [*const usb_sys::InterfaceDescriptor; 1] = [
    &KEYBOARD_INTERFACE_DESCRIPTOR,
];

const KEYBOARD_ENDPOINT_DESCRIPTOR: usb_sys::EndpointDescriptor = usb_sys::EndpointDescriptor {
    length: core::mem::size_of::<usb_sys::EndpointDescriptor>() as u8,
    descriptor_type: 0x05,
    endpoint_address: 0x03 | 0x80,
    attributes: 0x03,
    max_packet_size: 8,
    interval: 0x01
};

const KEYBOARD_ENDPOINT_DESCRIPTORS: [*const usb_sys::EndpointDescriptor; 1] = [
    &KEYBOARD_ENDPOINT_DESCRIPTOR,
];

const KEYBOARD_HID_DESCRIPTOR: usb_sys::HIDDescriptor = usb_sys::HIDDescriptor {
    length: core::mem::size_of::<usb_sys::HIDDescriptor>() as u8,
    descriptor_type: 0x21,
    hid_version: 0x0111,
    country_code: 0,
    num_child_descriptors: 1,
    child_descriptor_type: 0x22,
    child_descriptor_length: KEYBOARD_REPORT_DESCRIPTOR.len() as u16,
};

const KEYBOARD_HID_DESCRIPTORS: [*const usb_sys::HIDDescriptor; 1] = [
    &KEYBOARD_HID_DESCRIPTOR,
];

fn main_impl() -> ! {
    unsafe {
        *DDRB |= 1;
        *DDRD |= 1 << 5;

        *PORTB |= 1;
    }

    let config = usb_sys::USBConfig {
	device_descriptor: &KEYBOARD_DEVICE_DESCRIPTOR,
	configuration_descriptors: KEYBOARD_CONFIG_DESCRIPTORS.as_ptr(),
        interface_descriptors: KEYBOARD_INTERFACE_DESCRIPTORS.as_ptr(),
        endpoint_descriptors: KEYBOARD_ENDPOINT_DESCRIPTORS.as_ptr(),
        hid_descriptors: KEYBOARD_HID_DESCRIPTORS.as_ptr(),
        report_descriptor_length: KEYBOARD_REPORT_DESCRIPTOR.len() as u8,
        report_descriptor: KEYBOARD_REPORT_DESCRIPTOR.as_ptr(),
    };
    unsafe {
        usb_sys::usb_init(&config as *const usb_sys::USBConfig);
    }
    while unsafe { usb_sys::usb_get_state() } != usb_sys::USBState::Attached {
        unsafe {
            *PORTB ^= 1;
            *PORTD ^= 1 << 5;
        }
        delay_ms(1000);
    }

    unsafe {
        *PORTB &= !1;
        *PORTD &= !(1 << 5);
    }

    loop {}
}

fn delay_ms(milliseconds: usize) {
    const CYCLES_PER_MILLISECOND: usize = 1600;
    for _ in 0..milliseconds {
        for _ in 0..CYCLES_PER_MILLISECOND {
            unsafe { core::arch::asm!("nop") }
        }
    }
}

#[link(name = "c", kind = "static")]
extern "C" { }

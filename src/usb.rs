/// descriptor implements a collection of functions
/// which are required on the raw descriptor types.
macro_rules! descriptor {
    (name = $descriptor_name:ident, descriptor_type = $descriptor_type:expr) => {
        impl $descriptor_name {
            const fn length() -> usize {
                // 2 extra bytes are length and descriptor_type
                core::mem::size_of::<Self>() + 2
            }

            const fn descriptor_type() -> u8 {
                $descriptor_type
            }

            pub fn serialize(&self) -> [u8; Self::length()] {
                let mut bytes = [0; Self::length()];
                bytes[0] = Self::length() as u8;
                bytes[1] = Self::descriptor_type();
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        self as *const Self as *const u8,
                        bytes.as_mut_ptr().offset(2),
                        core::mem::size_of::<Self>(),
                    );
                }
                bytes
            }
        }
    };
}

#[repr(C)]
pub struct DeviceDescriptor {
    usb_version: u16,
    device_class: u8,
    device_subclass: u8,
    device_protocol: u8,
    max_packet_size: u8,
    vendor_id: u16,
    product_id: u16,
    device_version: u16,
    manufacturer_string_index: u8,
    product_string_index: u8,
    serial_number_string_index: u8,
    num_configurations: u8,
}

#[repr(C)]
pub struct ConfigurationDescriptor {
    total_length: u16,
    num_interfaces: u8,
    configuration_value: u8,
    configuration_string_index: u8,
    attributes: u8,
    max_power: u8,
}

#[repr(C)]
pub struct InterfaceDescriptor {
    interface_number: u8,
    alternate_setting: u8,
    num_endpoints: u8,
    interface_class: u8,
    interface_subclass: u8,
    interface_protocol: u8,
    interface_string_index: u8,
}

#[repr(C)]
pub struct EndpointDescriptor {
    endpoint_address: u8,
    attributes: u8,
    max_packet_size: u16,
    interval: u8,
}

#[repr(C)]
pub struct HIDDescriptor {
    hid_version: u16,
    country_code: u8,
    num_child_descriptors: u8,
    // TODO: there can be N of these, not exactly 1.
    child_descriptor_type: u8,
    child_descriptor_length: u16,
}

descriptor!(name = DeviceDescriptor, descriptor_type = 0x01);
descriptor!(name = ConfigurationDescriptor, descriptor_type = 0x02);
descriptor!(name = InterfaceDescriptor, descriptor_type = 0x04);
descriptor!(name = EndpointDescriptor, descriptor_type = 0x05);
descriptor!(name = HIDDescriptor, descriptor_type = 0x21);

pub static DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
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

pub static CONFIG_DESCRIPTOR: ConfigurationDescriptor = ConfigurationDescriptor {
    total_length: (ConfigurationDescriptor::length() as u16
        + InterfaceDescriptor::length() as u16
        + EndpointDescriptor::length() as u16
        + HIDDescriptor::length() as u16),
    num_interfaces: 1,
    configuration_value: 1,
    configuration_string_index: 0,
    attributes: 0xC0,
    max_power: 50,
};

pub static KEYBOARD_INTERFACE_DESCRIPTOR: InterfaceDescriptor = InterfaceDescriptor {
    interface_number: 0,
    alternate_setting: 0,
    num_endpoints: 1,
    interface_class: 0x03,    // interface class for HIDdescriptor
    interface_subclass: 0x01, // boot subclass, because this is a keyboard :^)
    interface_protocol: 0x01, // protocol for keyboard
    interface_string_index: 0,
};

pub static KEYBOARD_ENDPOINT_DESCRIPTOR: EndpointDescriptor = EndpointDescriptor {
    endpoint_address: 0x03 | 0x80,
    attributes: 0x03,
    max_packet_size: 8,
    interval: 0x01,
};

pub static KEYBOARD_HID_DESCRIPTOR: HIDDescriptor = HIDDescriptor {
    hid_version: 0x0111,
    country_code: 0,
    num_child_descriptors: 1,
    child_descriptor_type: 0x22,
    child_descriptor_length: 63, // sizeof(keyboard_report_descriptor),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_descriptor_equivalent() {
        let expected: [u8; DeviceDescriptor::length()] =
            [18, 1, 0, 2, 0, 0, 0, 32, 237, 254, 1, 0, 0, 1, 0, 0, 0, 1];
        let serialized = DEVICE_DESCRIPTOR.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_config_descriptor_equivalent() {
        let expected: [u8; ConfigurationDescriptor::length()] = [10, 2, 37, 0, 1, 1, 0, 192, 50, 0];
        let serialized = CONFIG_DESCRIPTOR.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_interface_descriptor_equivalent() {
        let expected = [9, 4, 0, 0, 1, 3, 1, 1, 0];
        let serialized = KEYBOARD_INTERFACE_DESCRIPTOR.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_endpoint_descriptor_equivalent() {
        let expected = [8, 5, 131, 3, 8, 0, 1, 0];
        let serialized = KEYBOARD_ENDPOINT_DESCRIPTOR.serialize();
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_hid_descriptor_equivalent() {
        let expected = [10, 33, 17, 1, 0, 1, 34, 0, 63, 0];
        let serialized = KEYBOARD_HID_DESCRIPTOR.serialize();
        assert_eq!(serialized, expected);
    }
}

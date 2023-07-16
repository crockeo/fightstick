#[repr(C)]
pub struct DeviceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub usb_version: u16,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub max_packet_size: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub manufacturer_string_index: u8,
    pub product_string_index: u8,
    pub serial_number_string_index: u8,
    pub num_configurations: u8,
}

#[repr(C)]
pub struct ConfigurationDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub total_length: u16,
    pub num_interfaces: u8,
    pub configuration_value: u8,
    pub configuration_string_index: u8,
    pub attributes: u8,
    pub max_power: u8,
}

#[repr(C)]
pub struct InterfaceDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub interface_number: u8,
    pub alternate_setting: u8,
    pub num_endpoints: u8,
    pub interface_class: u8,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
    pub interface_string_index: u8,
}

#[repr(C)]
pub struct EndpointDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub endpoint_address: u8,
    pub attributes: u8,
    pub max_packet_size: u16,
    pub interval: u8,
}

#[repr(C)]
pub struct StringDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub string: [u16],
}

#[repr(C)]
pub struct HIDDescriptor {
    pub length: u8,
    pub descriptor_type: u8,
    pub hid_version: u16,
    pub country_code: u8,
    pub num_child_descriptors: u8,
    // TODO: there can be N of these, not exactly 1.
    pub child_descriptor_type: u8,
    pub child_descriptor_length: u16,
}

#[repr(C)]
pub struct USBConfig {
    pub device_descriptor: *const DeviceDescriptor,
    pub configuration_descriptors: *const *const ConfigurationDescriptor,
    pub interface_descriptors: *const *const InterfaceDescriptor,
    pub endpoint_descriptors: *const *const EndpointDescriptor,
    pub hid_descriptors: *const *const HIDDescriptor,

    pub report_descriptor_length: u8,
    pub report_descriptor: *const u8,
}

#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(isize)]
pub enum USBState {
    Unknown = 0,
    Disconnected = 1,
    Attached = 2,
}

#[link(name = "usb", kind = "static")]
extern "C" {
    pub fn usb_init(config: *const USBConfig) -> isize;
    pub fn usb_get_state() -> USBState;
    pub fn usb_send() -> isize;
}

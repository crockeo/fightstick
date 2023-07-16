use core::ffi::c_int;

#[repr(C)]
pub struct DeviceDescriptor {
    length: u8,
    descriptor_type: u8,
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
    length: u8,
    descriptor_type: u8,
    total_length: u16,
    num_interfaces: u8,
    configuration_value: u8,
    configuration_string_index: u8,
    attributes: u8,
    max_power: u8,
}

#[repr(C)]
pub struct InterfaceDescriptor {
    length: u8,
    descriptor_type: u8,
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
    length: u8,
    descriptor_type: u8,
    endpoint_address: u8,
    attributes: u8,
    max_packet_size: u16,
    interval: u8,
}

#[repr(C)]
pub struct StringDescriptor {
    length: u8,
    descriptor_type: u8,
    string: [u16],
}

#[repr(C)]
pub struct HIDDescriptor {
    length: u8,
    descriptor_type: u8,
    hid_version: u16,
    country_code: u8,
    num_child_descriptors: u8,
    // TODO: there can be N of these, not exactly 1.
    child_descriptor_type: u8,
    child_descriptor_length: u16,
}

#[repr(C)]
pub struct USBConfig {
    device_descriptor: *const DeviceDescriptor,
    configuration_descriptors: *const *const ConfigurationDescriptor,
    interface_descriptors: *const *const InterfaceDescriptor,
    endpoint_descriptors: *const *const EndpointDescriptor,
    hid_descriptors: *const *const HIDDescriptor,

    report_descriptor_length: u8,
    report_descriptor: *const u8,
}

#[repr(C)]
pub enum USBState {
    Unknown,
    Disconnected,
    Attached,
}

extern "C" {
    fn usb_init(config: *const USBConfig) -> c_int;
    fn usb_send() -> c_int;
}

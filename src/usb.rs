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

pub static DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor{
    length: core::mem::size_of::<DeviceDescriptor>() as u8,
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

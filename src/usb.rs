#[repr(C)]
pub struct Descriptor {
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

impl Descriptor {
    const fn length() -> u8 {
        // 2 extra bytes are length and descriptor_type
        core::mem::size_of::<Self>() as u8 + 2
    }

    const fn descriptor_type() -> u8 {
        0x01
    }

    pub fn serialize(&self) -> [u8; Self::length() as usize] {
        let mut bytes = [0; Self::length() as usize];
        bytes[0] = Self::length();
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

pub static DEVICE_DESCRIPTOR: Descriptor = Descriptor {
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

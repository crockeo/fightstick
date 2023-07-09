#pragma once

typedef struct DeviceDescriptor {
  uint8_t length;
  uint8_t descriptor_type;
  uint16_t usb_version;
  uint8_t device_class;
  uint8_t device_subclass;
  uint8_t device_protocol;
  uint8_t max_packet_size;
  uint16_t vendor_id;
  uint16_t product_id;
  uint16_t device_version;
  uint8_t manufacturer_string_index;
  uint8_t product_string_index;
  uint8_t serial_number_string_index;
  uint8_t num_configurations;
} DeviceDescriptor;

typedef struct ConfigurationDescriptor {
  uint8_t length;
  uint8_t descriptor_type;
  uint16_t total_length;
  uint8_t num_interfaces;
  uint8_t configuration_value;
  uint8_t configuration_string_index;
  uint8_t attributes;
  uint8_t max_power;
} ConfigurationDescriptor;

typedef struct InterfaceDescriptor {
  uint8_t length;
  uint8_t descriptor_type;
  uint8_t interface_number;
  uint8_t alternate_setting;
  uint8_t num_endpoints;
  uint8_t interface_class;
  uint8_t interface_subclass;
  uint8_t interface_protocol;
  uint8_t interface_string_index;
} InterfaceDescriptor;

typedef struct EndpointDescriptor {
  uint8_t length;
  uint8_t descriptor_type;
  uint8_t endpoint_address;
  uint8_t attributes;
  uint16_t max_packet_size;
  uint8_t interval;
} EndpointDescriptor;

typedef struct StringDescriptor {
  uint8_t length;
  uint8_t descriptor_type;
  uint16_t string[];
} StringDescriptor;

#pragma once

#include <stdbool.h>
#include <stdint.h>

#include "descriptor.h"

typedef struct {
    DeviceDescriptor device_descriptor;
    const ConfigurationDescriptor** configuration_descriptors;
    const InterfaceDescriptor** interface_descriptors;
    const EndpointDescriptor** endpoint_descriptors;
    const HIDDescriptor** hid_descriptors;

    // TODO: get rid of this special case...
    uint8_t report_descriptor_length;
    const uint8_t* report_descriptor;
} usb_config_t;

int usb_init(const usb_config_t* usb_config);

typedef enum {
    USB_STATE_UNKNOWN,
    USB_STATE_DISCONNECTED,
    USB_STATE_ATTACHED,
} usb_state_t;

extern volatile usb_state_t usb_state;
extern volatile uint8_t keyboard_pressed_keys[6];

int usb_send();

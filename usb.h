#pragma once

typedef int usb_state_t;

#define USB_DEVICE_STATE_UNKNOWN 0
#define USB_DEVICE_STATE_RESET 1

volatile usb_state_t* get_usb_device_state();

void usb_init();

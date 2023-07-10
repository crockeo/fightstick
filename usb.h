#pragma once

#include <stdbool.h>
#include <stdint.h>

typedef enum {
    USB_STATE_UNKNOWN,
    USB_STATE_DISCONNECTED,
    USB_STATE_ATTACHED,
} usb_state_t;

extern volatile usb_state_t usb_state;
extern volatile uint8_t keyboard_pressed_keys[6];

int usb_init();
int usb_send();

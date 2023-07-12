#define F_CPU 16000000

#include <avr/io.h>
#include <avr/pgmspace.h>
#include <stdlib.h>
#include <string.h>
#include <util/delay.h>

#include "hal.h"
#include "keys.h"
#include "usb.h"

#define BUTTON_COUNT 10

// TODO(crockeo): make this into a struct, instead of a series of bytes.
// and that also means finding the spec which defines this thing...
static const uint8_t keyboard_report_descriptor[] PROGMEM = {
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
    0xC0   // End collection
};

static const DeviceDescriptor KEYBOARD_DEVICE_DESCRIPTOR PROGMEM = {
    .length = sizeof(DeviceDescriptor),
    .descriptor_type = 1,
    .usb_version = 0x0200,
    .device_class = 0,
    .device_subclass = 0,
    .device_protocol = 0,
    .max_packet_size = 32,
    .vendor_id = 0xfeed,
    .product_id = 0x0001,
    .device_version = 0x0100,
    .manufacturer_string_index = 0,
    .product_string_index = 0,
    .serial_number_string_index = 0,
    .num_configurations = 1,
};

static const ConfigurationDescriptor KEYBOARD_CONFIG_DESCRIPTOR PROGMEM = {
    .length = sizeof(ConfigurationDescriptor),
    .descriptor_type = 2,
    .total_length = (
	sizeof(ConfigurationDescriptor)
	+ sizeof(InterfaceDescriptor)
	+ sizeof(EndpointDescriptor)
	+ sizeof(HIDDescriptor)
    ),
    .num_interfaces = 1,
    .configuration_value = 1,
    .configuration_string_index = 0,
    .attributes = 0xC0,
    .max_power = 50,
};

static const ConfigurationDescriptor* KEYBOARD_CONFIG_DESCRIPTORS[] = {
    &KEYBOARD_CONFIG_DESCRIPTOR,
};

static const InterfaceDescriptor KEYBOARD_INTERFACE_DESCRIPTOR PROGMEM = {
    .length = sizeof(InterfaceDescriptor),
    .descriptor_type = 4,
    .interface_number = 0,
    .alternate_setting = 0,
    .num_endpoints = 1,
    .interface_class = 0x03, // interface class for HIDdescriptor
    .interface_subclass = 0x01,  // boot subclass, because this is a keyboard :^)
    .interface_protocol = 0x01,  // protocol for keyboard
    .interface_string_index = 0,
};

static const InterfaceDescriptor* KEYBOARD_INTERFACE_DESCRIPTORS[] = {
    &KEYBOARD_INTERFACE_DESCRIPTOR,
};

static const EndpointDescriptor KEYBOARD_ENDPOINT_DESCRIPTOR PROGMEM = {
    .length = sizeof(EndpointDescriptor),
    .descriptor_type = 0x05,
    .endpoint_address = 0x03 | 0x80,
    .attributes = 0x03,
    .max_packet_size = 8,
    .interval = 0x01
};

static const EndpointDescriptor* KEYBOARD_ENDPOINT_DESCRIPTORS[] = {
    &KEYBOARD_ENDPOINT_DESCRIPTOR,
};

static const HIDDescriptor KEYBOARD_HID_DESCRIPTOR PROGMEM = {
    .length = sizeof(HIDDescriptor),
    .descriptor_type = 0x21,
    .hid_version = 0x0111,
    .country_code = 0,
    .num_child_descriptors = 1,
    .child_descriptor_type = 0x22,
    .child_descriptor_length = sizeof(keyboard_report_descriptor),
};

static const HIDDescriptor* KEYBOARD_HID_DESCRIPTORS[] = {
    &KEYBOARD_HID_DESCRIPTOR,
};

static const usb_config_t USB_CONFIG = {
    .device_descriptor = &KEYBOARD_DEVICE_DESCRIPTOR,
    .configuration_descriptors = KEYBOARD_CONFIG_DESCRIPTORS,
    .interface_descriptors = KEYBOARD_INTERFACE_DESCRIPTORS,
    .endpoint_descriptors = KEYBOARD_ENDPOINT_DESCRIPTORS,
    .hid_descriptors = KEYBOARD_HID_DESCRIPTORS,

    .report_descriptor = keyboard_report_descriptor,
    .report_descriptor_length = sizeof(keyboard_report_descriptor),
};

typedef struct {
    pin_t pin;
    uint8_t scancode;
} PinButton;

static PinButton buttons[BUTTON_COUNT] = {
    {PIN_D2, KEY_A},
    {PIN_D3, KEY_S},
    {PIN_D4, KEY_D},
    {PIN_D6, KEY_W},
    {PIN_D7, KEY_J},
    {PIN_D8, KEY_K},
    {PIN_D9, KEY_L},
    {PIN_D16, KEY_U},
    {PIN_D14, KEY_I},
    {PIN_D15, KEY_O},
};

void turn_on_leds() {
  PORTB &= ~(1 << PB0);
  PORTD &= ~(1 << PD5);
}

void turn_off_leds() {
  PORTB |= (1 << PB0);
  PORTD |= (1 << PD5);
}

int main(int argc, char** argv) {
    usb_init(&USB_CONFIG);

    // Set LEDs to output.
    DDRB |= (1 << PB0);
    DDRD |= (1 << PD5);

    // Set buttons to input.
    DDRD &= (~(1 << PD0) & ~(1 << PD1) & ~(1 << PD4));

    while (usb_state != USB_STATE_ATTACHED) {
	turn_off_leds();
	_delay_ms(100);
	turn_on_leds();
	_delay_ms(100);
    }

    PORTD = 0; // push nothing out of port 0 to start with...
    for (int i = 0; i < BUTTON_COUNT; i++) {
	set_pull_up(buttons[i].pin);
    }

    while (true) {
	bool leds_on = false;
	for (int i = 0; i < 6; i++) {
	    keyboard_pressed_keys[i] = 0;
	}

	int keyboard_index = 0;
	for (int i = 0; i < BUTTON_COUNT; i++) {
	    if (keyboard_index >= 6) {
		break;
	    }

	    if (is_pin_low(buttons[i].pin)) {
		leds_on = true;
		keyboard_pressed_keys[keyboard_index] = buttons[i].scancode;
		keyboard_index++;
	    }
	}

	if (leds_on) {
	    turn_on_leds();
	} else {
	    turn_off_leds();
	}
	usb_send();
    }
}

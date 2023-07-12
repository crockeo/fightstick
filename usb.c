#include "usb.h"

#define F_CPU 16000000

#include <avr/interrupt.h>
#include <avr/io.h>
#include <avr/pgmspace.h>
#include <util/delay.h>

#include "descriptor.h"

#define DESCRIPTOR_REQUEST_DEVICE 0x01
#define DESCRIPTOR_REQUEST_CONFIGURATION 0x02
#define DESCRIPTOR_REQUEST_HID 0x21
#define DESCRIPTOR_REQUEST_REPORT 0x22

// General USB request codes.
#define GET_STATUS 0x00
#define CLEAR_FEATURE 0x01
#define SET_FEATURE 0x03
#define SET_ADDRESS 0x05
#define GET_DESCRIPTOR 0x06
#define GET_CONFIGURATION 0x08
#define SET_CONFIGURATION 0x09
#define GET_INTERFACE 0x0A
#define SET_INTERFACE 0x0B

// HID class-specific request codes.
#define GET_REPORT 0x01
#define GET_IDLE 0x02
#define GET_PROTOCOL 0x03
#define SET_REPORT 0x09
#define SET_IDLE 0x0A
#define SET_PROTOCOL 0x0B

#define KEYBOARD_ENDPOINT_NUM 3  // The second endpoint is the HID endpoint


volatile usb_state_t usb_state = USB_STATE_UNKNOWN;

volatile uint8_t keyboard_pressed_keys[6] = {0, 0, 0, 0, 0, 0};

static uint16_t keyboard_idle_value =
    125;  // HID Idle setting, how often the device resends unchanging reports,
          // we are using a scaling of 4 because of the register size
static uint8_t current_idle =
    0;  // Counter that updates based on how many SOFE interrupts have occurred
static uint8_t this_interrupt =
    0;  // This is not the best way to do it, but it
        // is much more readable than the alternative

uint8_t keyboard_protocol = 0;

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
    .length = 9,
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

static const InterfaceDescriptor KEYBOARD_INTERFACE_DESCRIPTOR PROGMEM = {
    .length = 9,
    .descriptor_type = 4,
    .interface_number = 0,
    .alternate_setting = 0,
    .num_endpoints = 1,
    .interface_class = 0x03, // interface class for HIDdescriptor
    .interface_subclass = 0x01,  // boot subclass, because this is a keyboard :^)
    .interface_protocol = 0x01,  // protocol for keyboard
    .interface_string_index = 0,
};

static const EndpointDescriptor KEYBOARD_ENDPOINT_DESCRIPTOR PROGMEM = {
    .length = 7,
    .descriptor_type = 0x05,
    .endpoint_address = KEYBOARD_ENDPOINT_NUM | 0x80,
    .attributes = 0x03,
    .max_packet_size = 8,
    .interval = 0x01
};

static const HIDDescriptor KEYBOARD_HID_DESCRIPTOR PROGMEM = {
    .length = 9,
    .descriptor_type = 0x21,
    .hid_version = 0x0111,
    .country_code = 0,
    .num_child_descriptors = 1,
    .child_descriptor_type = 0x22,
    .child_descriptor_length = sizeof(keyboard_report_descriptor),
};

int usb_init() {
  cli();  // Global Interrupt Disable

  UHWCON |= (1 << UVREGE);  // Enable USB Pads Regulator

  PLLCSR |= 0x12;  // Configure to use 16mHz oscillator

  while (!(PLLCSR & (1 << PLOCK)))
    ;  // Wait for PLL Lock to be achieved

  USBCON |=
      (1 << USBE) | (1 << OTGPADE);  // Enable USB Controller and USB power pads
  USBCON &= ~(1 << FRZCLK);          // Unfreeze the clock

  UDCON &= ~(1<<LSM);  // FULL SPEED MODE
  UDCON &= ~(1<<DETACH);  // Attach USB Controller to the data bus

  UDIEN |= (1 << EORSTE) |
           (1 << SOFE);  // Re-enable the EORSTE (End Of Reset) Interrupt so we
                         // know when we can configure the control endpoint
  usb_state = USB_STATE_DISCONNECTED;
  sei();  // Global Interrupt Enable
  return 0;
}

int usb_send() {
    if (usb_state != USB_STATE_ATTACHED) {
	return -1;
    }

  cli();
  UENUM = KEYBOARD_ENDPOINT_NUM;

  while (!(UEINTX & (1 << RWAL)))
    ;  // Wait for banks to be ready
  UEDATX = 0; // modifier
  UEDATX = 0; // reserved byte
  for (int i = 0; i < 6; i++) {
    UEDATX = keyboard_pressed_keys[i];
  }

  UEINTX = 0b00111010;
  current_idle = 0;
  sei();
  return 0;
}

ISR(USB_GEN_vect) {
  uint8_t udint_temp = UDINT;
  UDINT = 0;

  if (udint_temp & (1 << EORSTI)) {  // If end of reset interrupt
    // Configure Control Endpoint
    UENUM = 0;             // Select Endpoint 0, the default control endpoint
    UECONX = (1 << EPEN);  // Enable the Endpoint
    UECFG0X = 0;      // Control Endpoint, OUT direction for control endpoint
    UECFG1X |= 0x22;  // 32 byte endpoint, 1 bank, allocate the memory
    usb_state = USB_STATE_DISCONNECTED;

    if (!(UESTA0X &
          (1 << CFGOK))) {  // Check if endpoint configuration was successful
      return;
    }

    UERST = 1;  // Reset Endpoint
    UERST = 0;

    UEIENX =
        (1 << RXSTPE);  // Re-enable the RXSPTE (Receive Setup Packet) Interrupt
    return;
  }
  if ((udint_temp & (1 << SOFI)) && usb_state == USB_STATE_ATTACHED) {  // Check for Start Of Frame Interrupt and correct
                            // usb configuration, send keypress if a keypress
                            // event has not been sent through usb_send
    this_interrupt++;
    if (keyboard_idle_value &&
        (this_interrupt & 3) == 0) {  // Scaling by four, trying to save memory
      UENUM = KEYBOARD_ENDPOINT_NUM;
      if (UEINTX & (1 << RWAL)) {  // Check if banks are writable
        current_idle++;
        if (current_idle ==
            keyboard_idle_value) {  // Have we reached the idle threshold?
          current_idle = 0;
          UEDATX = 0; // modifier
          UEDATX = 0; // reserved byte
          for (int i = 0; i < 6; i++) {
            UEDATX = keyboard_pressed_keys[i];
          }
          UEINTX = 0b00111010;
        }
      }
    }
  }
}

int pause_tx() {
    UEINTX &= ~(1 << TXINI);
    // TODO(crockeo): find all of the places where this happens
    // and abstract them in some kind of in-lined like "wait for tx" function
    while ((UEINTX & (1 << TXINI)) == 0) {}
    if ((UEINTX & (1 << RXOUTI)) != 0) {
	return -1;
    }
    return 0;
}

int write_descriptor_part(uint8_t remaining_packet_length, uint8_t const* descriptor, uint8_t descriptor_length) {
    uint8_t packet_length = descriptor_length;
    if (packet_length > remaining_packet_length) {
	packet_length = remaining_packet_length;
    }
    for (int i = 0; i < packet_length; i++) {
	UEDATX = pgm_read_byte(descriptor + i);
    }
    return packet_length;
}

int write_descriptors(uint16_t request_length, uint8_t const* descriptors[], uint8_t descriptors_length) {
    if (request_length > 255) {
	request_length = 255;
    }
    uint8_t remaining_packet_length = 32;
    for (int i = 0; i < descriptors_length; i++) {
	uint8_t const* descriptor = descriptors[i];
	uint8_t descriptor_length = pgm_read_byte(descriptor);
	if (descriptor_length > request_length) {
	    descriptor_length = request_length;
	}
	request_length -= descriptor_length;

	while (descriptor_length > 0) {
	    uint8_t written = write_descriptor_part(remaining_packet_length, descriptor, descriptor_length);
	    descriptor += written;
	    descriptor_length -= written;
	    remaining_packet_length -= written;

	    if (remaining_packet_length == 0) {
		if (pause_tx() < 0) {
		    return -1;
		}
		remaining_packet_length = 32;
	    }
	}
    }
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int write_descriptor(uint16_t request_length, uint8_t const* descriptor, uint8_t descriptor_length) {
    // 255 bytes is the maximum packet size for USB.
    if (request_length > 255) {
	request_length = 255;
    }
    if (descriptor_length > request_length) {
	descriptor_length = request_length;
    }

    uint8_t descriptor_remaining = descriptor_length;
    while (descriptor_remaining > 0) {
	while ((UEINTX & (1 << TXINI)) == 0) {}
	if ((UEINTX & (1 << RXOUTI)) != 0) {
	    return -1;
	}

	uint8_t packet_size = descriptor_remaining;
	if (packet_size > 32) {
	    packet_size = 32;
	}

	for (int i = 0; i < packet_size; i++) {
	    UEDATX = pgm_read_byte(descriptor + i);
	}

	descriptor_remaining -= packet_size;
	descriptor += packet_size;
	UEINTX &= ~(1 << TXINI);
    }
    return descriptor_length;
}


int write_device_descriptor(uint16_t request_length) {
    return write_descriptor(
	request_length,
	(uint8_t const*)&KEYBOARD_DEVICE_DESCRIPTOR,
	sizeof(DeviceDescriptor)
    );
}

int write_configuration_descriptor(uint16_t request_length) {
    uint8_t const* descriptors[4] = {
	(uint8_t const*)&KEYBOARD_CONFIG_DESCRIPTOR,
	(uint8_t const*)&KEYBOARD_INTERFACE_DESCRIPTOR,
	(uint8_t const*)&KEYBOARD_HID_DESCRIPTOR,
	(uint8_t const*)&KEYBOARD_ENDPOINT_DESCRIPTOR,
    };
    return write_descriptors(
	request_length,
	descriptors,
	4
    );
}

int write_hid_report_descriptor(uint16_t request_length) {
    return write_descriptor(
	request_length,
	(uint8_t const*)&KEYBOARD_HID_DESCRIPTOR,
	sizeof(KEYBOARD_HID_DESCRIPTOR)
    );
}

int write_report_descriptor(uint16_t request_length) {
    // TODO: this isn't actually a normal descriptor
    // and so it can't use the pgm_read_byte(...) of the first element
    return write_descriptor(
	request_length,
	keyboard_report_descriptor,
	sizeof(keyboard_report_descriptor)
    );
}

typedef struct {
    uint8_t request_type;
    uint8_t request;
    uint16_t value;
    uint16_t index;
    uint16_t length;
} USBRequest;

int handle_usb_get_descriptor_request(USBRequest* request) {
    switch (request->value >> 8) {
    case DESCRIPTOR_REQUEST_DEVICE:
	write_device_descriptor(request->length);
	break;
    case DESCRIPTOR_REQUEST_CONFIGURATION:
	write_configuration_descriptor(request->length);
	break;
    case DESCRIPTOR_REQUEST_HID:
	write_hid_report_descriptor(request->length);
	break;
    case DESCRIPTOR_REQUEST_REPORT:
	write_report_descriptor(request->length);
	break;
    default:
	// Enable the endpoint and stall, the
	// descriptor does not exist
	PORTC = 0xFF;
	UECONX |= (1 << STALLRQ) | (1 << EPEN);
	return -1;
    }
    return 0;
}

int handle_set_configuration_request(USBRequest* request) {
    if (request->request_type != 0) {
	return 0;
    }

    usb_state = USB_STATE_ATTACHED;
    UEINTX &= ~(1 << TXINI);
    UENUM = KEYBOARD_ENDPOINT_NUM;
    UECONX = 1;
    UECFG0X = 0b11000001;  // EPTYPE Interrupt IN
    UECFG1X = 0b00000110;  // Dual Bank Endpoint, 8 Bytes, allocate memory
    UERST = 0x1E;          // Reset all of the endpoints
    UERST = 0;
    return 0;
}

int handle_set_address_request(USBRequest* request) {
    UEINTX &= ~(1 << TXINI);
    while ((UEINTX & (1 << TXINI)) == 0) { }
    UDADDR = request->value | (1 << ADDEN);  // Set the device address
    return 0;
}

int handle_get_configuration_request(USBRequest* request) {
    if (request->request_type != 0x80) {
	return 0;
    }

    while ((UEINTX & (1 << TXINI)) == 0) {}
    UEDATX = usb_state;
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int handle_get_status_request(USBRequest* request) {
    while ((UEINTX & (1 << TXINI)) == 0) {}
    UEDATX = 0;
    UEDATX = 0;
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int handle_get_report_request(USBRequest* request) {
    while ((UEINTX & (1 << TXINI)) == 0) {}
    UEDATX = 0; // modifier
    UEDATX = 0; // reserved byte
    for (int i = 0; i < 6; i++) {
	UEDATX = keyboard_pressed_keys
	    [i];  // According to the spec, this method of getting the
	// report is not used for device polling, although we
	// still have to implement the response
    }
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int handle_get_idle_request(USBRequest* request) {
    while ((UEINTX & (1 << TXINI)) == 0) {}
    UEDATX = keyboard_idle_value;
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int handle_get_protocol_request(USBRequest* request) {
    while ((UEINTX & (1 << TXINI)) == 0) {}
    UEDATX = keyboard_protocol;
    UEINTX &= ~(1 << TXINI);
    return 0;
}

int handle_set_report_request(USBRequest* request) {
    while (!(UEINTX & (1 << RXOUTI)))
	;  // This is the opposite of the TXINI one, we are waiting until
    // the banks are ready for reading instead of for writing
    UEINTX &= ~(1 << TXINI);  // Send ACK and clear TX bit
    UEINTX &= ~(1 << RXOUTI);
    return 0;
}

int handle_set_idle_request(USBRequest* request) {
    keyboard_idle_value = request->value;  //
    current_idle = 0;

    UEINTX &= ~(1 << TXINI);  // Send ACK and clear TX bit
    return 0;
}

int handle_set_protocol_request(USBRequest* request) {
    keyboard_protocol =
	request->value >> 8;  // Nobody cares what happens to this, arbitrary cast
    // from 16 bit to 8 bit doesn't matter

    UEINTX &= ~(1 << TXINI);  // Send ACK and clear TX bit
    return 0;
}

int handle_usb_request() {
    USBRequest request;
    for (int i = 0; i < sizeof(USBRequest); i++) {
	((uint8_t*)&request)[i] = UEDATX;
    }

    DDRC = 0xFF;
    UEINTX &= ~(
        (1 << RXSTPI) | (1 << RXOUTI) |
        (1 << TXINI));  // Handshake the Interrupts, do this after recording
                        // the packet because it also clears the endpoint banks

    // General USB requests.
    switch (request.request) {
    case GET_DESCRIPTOR:
	return handle_usb_get_descriptor_request(&request);
    case SET_CONFIGURATION:
	return handle_set_configuration_request(&request);
    case SET_ADDRESS:
	return handle_set_address_request(&request);
    case GET_CONFIGURATION:
	return handle_get_configuration_request(&request);
    case GET_STATUS:
	return handle_get_status_request(&request);
    }

    // All class-specific requests have index == the interface number.
    // Our interface number happens to be 0 so...easy hack.
    if (request.index != 0) {
	return -1;
    }

    if (request.request_type == 0b10100001) {
	switch (request.request) {
	case GET_REPORT:
	    return handle_get_report_request(&request);
	case GET_IDLE:
	    return handle_get_idle_request(&request);
	case GET_PROTOCOL:
	    return handle_get_protocol_request(&request);
	}
    }

    if (request.request_type == 0b00100001) {
	switch (request.request) {
	case SET_REPORT:
	    return handle_set_report_request(&request);
	case SET_IDLE:
	    return handle_set_idle_request(&request);
	case SET_PROTOCOL:
	    return handle_set_protocol_request(&request);
	}
    }
    return -1;
}

ISR(USB_COM_vect) {
  UENUM = 0;
  if (UEINTX & (1 << RXSTPI)) {
      if (handle_usb_request() < 0) {
	  // TODO: errors
      }
      return;
  }
  PORTC = 0xFF;
  UECONX |= (1 << STALLRQ) |
      (1 << EPEN);  // The host made an invalid request or there was an
  // error with one of the request parameters
}

#define F_CPU 2000000UL

#include <avr/interrupt.h>
#include <avr/io.h>
#include <stdbool.h>
#include <time.h>
#include <util/delay.h>

#include "descriptor.h"
#include "usb.h"

#define USB_REQUEST_GET_STATUS 0x0
#define USB_REQUEST_CLEAR_FEATURE 0x1
#define USB_REQUEST_SET_FEATURE 0x3
#define USB_REQUEST_SET_ADDRESS 0x5
#define USB_REQUEST_GET_DESCRIPTOR 0x6
#define USB_REQUEST_SET_DESCRIPTOR 0x7
#define USB_REQUEST_GET_CONFIGURATION 0x8
#define USB_REQUEST_SET_CONFIGURATION 0x9
#define USB_REQUEST_GET_INTERFACE 0xA
#define USB_REQUEST_SET_INTERFACE 0xB

#define USB_DESCRIPTOR_TYPE_DEVICE 0x1
#define USB_DESCRIPTOR_TYPE_CONFIGURATION 0x2
#define USB_DESCRIPTOR_TYPE_STRING 0x3
#define USB_DESCRIPTOR_TYPE_INTERFACE 0x4
#define USB_DESCRIPTOR_TYPE_ENDPOINT 0x5

static const DeviceDescriptor KEYBOARD_DESCRIPTOR = {
  .length = sizeof(DeviceDescriptor),
  .descriptor_type = USB_DESCRIPTOR_TYPE_DEVICE,
  .usb_version = 0x2,
  .device_class = 0x0,
  .device_subclass = 0x0,
  .device_protocol = 0x0,
  .max_packet_size = 32,
  .vendor_id = 0x0,
  .product_id = 0x0,
  .device_version = 0x1,
  .manufacturer_string_index = 0x0,
  .product_string_index = 0x0,
  .serial_number_string_index = 0x0,
  .num_configurations = 0x1,
};

static const ConfigurationDescriptor KEYBOARD_CONFIGURATION_DESCRIPTOR = {
  .length = sizeof(ConfigurationDescriptor),
  .descriptor_type = USB_DESCRIPTOR_TYPE_CONFIGURATION,
  .total_length = sizeof(ConfigurationDescriptor) + sizeof(InterfaceDescriptor) + sizeof(EndpointDescriptor),
  .num_interfaces = 1,
  .configuration_value = 1,
  .configuration_string_index = 0x0,
  .attributes = 0xC0,
  .max_power = 50,
};

static const InterfaceDescriptor KEYBOARD_INTERFACE_DESCRIPTOR = {
  .length = sizeof(InterfaceDescriptor),
  .descriptor_type = USB_DESCRIPTOR_TYPE_INTERFACE,
  .interface_number = 0,
  .alternate_setting = 0,
  .num_endpoints = 1,
  .interface_class = 0x3,
  .interface_subclass = 0x1,
  .interface_protocol = 0x1,
  .interface_string_index = 0x0,
};

static const EndpointDescriptor KEYBOARD_ENDPOINT_DESCRIPTOR = {
  .length = sizeof(EndpointDescriptor),
  .descriptor_type = USB_DESCRIPTOR_TYPE_ENDPOINT,
  .endpoint_address = 0x81, // TODO: why this magic number?
  .attributes = 0x3,
  .max_packet_size = 8,
  .interval = 1,
};

// 1 -> 12
// JP6 = RAW, GND, RESET, VCC, A3, A2, A1, A0, SCK, MISO, MOSI, D10
// JP7 = D9, D8, D7, D6, D5, D4, D3, D2, GND GND, RXI, TXO


// RX_LED = PB0 = D17 = SS
// TX_LED = D30 = PD5

void disable_interrupts() { cli(); }
void enable_interrupts() { sei(); }

void turn_on_leds() {
  PORTD &= ~(1 << PD5);
  PORTB &= ~(1 << PB0);
}

void turn_off_leds() {
  PORTD |= (1 << PD5);
  PORTB |= (1 << PB0);
}

int main() {
  disable_interrupts();
  usb_init();
  enable_interrupts();

  PORTD = 0; // push nothing out of port 0 to start with...
  DDRD &= ~(1 << PD1); // set PD1 to be input
  PORTD |= (1 << PD1) | (1 << PD0) | (1 << PD4); // set PD1 to be pull-up
  while (true) {
    int left_pressed = (PIND & (1 << PD1)) == 0;
    int down_pressed = (PIND & (1 << PD0)) == 0;
    int right_pressed = (PIND & (1 << PD4)) == 0;

    if (left_pressed || down_pressed || right_pressed) {
      turn_on_leds();
    } else {
      turn_off_leds();
    }
  }
}

typedef struct USBRequest {
  uint8_t request_type;
  uint8_t request;
  uint16_t value;
  uint16_t index;
  uint16_t length;
} USBRequest;

int write_descriptor(uint8_t* descriptor, uint8_t descriptor_size) {
  while (descriptor_size > 0) {
    while ((UEINTX & (1 << TXINI)) == 0) {}
    if ((UEINTX & (1 << RXOUTI)) != 0) {
      return -1;
    }

    uint8_t bytes_to_send =
      descriptor_size > 32
      ? 32
      : descriptor_size;

    for (int i = 0; i < bytes_to_send; i++) {
      UEDATX = descriptor[i];
    }

    descriptor_size -= bytes_to_send;
    descriptor += bytes_to_send;
    UEINTX &= ~(1 << TXINI);
  }
  return 0;
}

int handle_get_descriptor_request(USBRequest usb_request) {
  if (usb_request.value == USB_DESCRIPTOR_TYPE_DEVICE) {
    write_descriptor((uint8_t*)&KEYBOARD_DESCRIPTOR, sizeof(DeviceDescriptor));
    return 0;
  }

  if (usb_request.value == USB_DESCRIPTOR_TYPE_CONFIGURATION) {
    write_descriptor((uint8_t*)&KEYBOARD_CONFIGURATION_DESCRIPTOR, sizeof(ConfigurationDescriptor));
    write_descriptor((uint8_t*)&KEYBOARD_INTERFACE_DESCRIPTOR, sizeof(InterfaceDescriptor));
    write_descriptor((uint8_t*)&KEYBOARD_ENDPOINT_DESCRIPTOR, sizeof(EndpointDescriptor));
    return 0;
  }

  return -1;
}

void usb_request() {
  // Reference:
  // https://www.beyondlogic.org/usbnutshell/usb6.shtml
  USBRequest usb_request;
  uint8_t* usb_request_ptr = (uint8_t*)&usb_request;
  for (int i = sizeof(USBRequest) - 1; i >= 0; i--) {
    usb_request_ptr[i] = UEDATX;
  }

  /* if (usb_request.request == USB_REQUEST_GET_DESCRIPTOR) { */
  /*   if (handle_get_descriptor_request(usb_request) != 0) { */
  /*     // TODO: error handling? */
  /*   } */
  /* } */

  /* if (usb_request.request == USB_REQUEST_SET_CONFIGURATION && usb_request.request_type == 0) { */
  /*   // TODO: put device into address mode */
  /* } */

  /* if (usb_request.request == USB_REQUEST_SET_ADDRESS) { */
  /* } */

  /* if (usb_request.request == USB_REQUEST_GET_CONFIGURATION) { */
  /* } */

  /* if (usb_request.request == USB_REQUEST_GET_STATUS) { */
  /* } */
}

ISR(USB_COM_vect) {
  if ((UENUM & (1 << RXSTPI)) != 0) {
    UENUM &= ~(1 << RXSTPI);
    usb_request();
  }
}

int usb_reset() {
  UENUM = 0;
  UECONX = (1 << EPEN);
  UECFG0X = 0;
  UECFG1X = 34;
  if ((UESTA0X & (1 << CFGOK)) == 0) {
    return -1;
  }
  return 0;
}

ISR(USB_GEN_vect) {
  if ((USBINT & (1 << VBUSTI)) != 0) {
    USBINT &= ~(1 << VBUSTI);
    // TODO: what to do here?
  }

  if ((UDINT & (1 << EORSTI)) != 0) {
    UDINT &= ~(1 << EORSTI);
    usb_reset();
  }

  // TODO: when i want to send frames
  /* if ((UDINT & (1 << SOFI)) != 0) { */
  /*   UDINT &= ~(1 << SOFI); */
  /*   usb_frame(); */
  /* } */
}

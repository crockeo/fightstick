#include "usb.h"

#include <avr/io.h>

volatile usb_state_t USB_DEVICE_STATE = USB_DEVICE_STATE_UNKNOWN;

volatile usb_state_t* get_usb_device_state() {
  return &USB_DEVICE_STATE;
}

void usb_init() {
  // Reset USB interface
  USBCON &= ~(0x01 << USBE);
  USBCON |= (0x01 << USBE);

  // Enable USB pad regulator.
  UHWCON |= (1 << UVREGE);

  // Unfreeze USB clock.
  USBCON &= ~(1 << FRZCLK);

  // Enable VBUS pad.
  USBCON |= (1 << OTGPADE);

  // Make the device go in full speed mode.
  UDCON &= (0x01 << LSM);

  // Enable interrupts.
  USBCON |= (1 << VBUSTE); // VBUS
  UDIEN |= (1 << EORSTE); // End of reset interrupts.
  // TODO: when i want to start sending frames

  USB_DEVICE_STATE = USB_DEVICE_STATE_RESET;
}

void usb_poll() {
  // TODO: handle poll...
}

#define F_CPU 16000000

#include <avr/io.h>
#include <stdlib.h>
#include <string.h>
#include <util/delay.h>

#include "hal.h"
#include "keys.h"
#include "usb.h"

#define BUTTON_COUNT 10

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
    usb_init();

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

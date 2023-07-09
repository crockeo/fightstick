#define F_CPU 16000000

#include <avr/io.h>
#include <string.h>
#include <util/delay.h>

#include "keys.h"
#include "usb.h"

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

    while (!get_usb_config_status()) {
	turn_off_leds();
	_delay_ms(100);
	turn_on_leds();
	_delay_ms(100);
    }

    PORTD = 0; // push nothing out of port 0 to start with...
    DDRD &= ~(1 << PD1); // set PD1 to be input
    PORTD |= (1 << PD1) | (1 << PD0) | (1 << PD4); // set PD1 to be pull-up
    while (true) {
	int left_pressed = (PIND & (1 << PD1)) == 0;
	int down_pressed = (PIND & (1 << PD0)) == 0;
	int right_pressed = (PIND & (1 << PD4)) == 0;

	if (left_pressed || down_pressed || right_pressed) {
	    turn_on_leds();
	    for (int i = 0; i < 6; i++) {
		send_keypress(KEY_A, 0);
	    }
	} else {
	    turn_off_leds();
	}
    }
}

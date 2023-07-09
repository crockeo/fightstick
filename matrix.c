/*
 * matrix.c
 */
#define F_CPU 16000000
#include "matrix.h"
#include "layout.h"
#include "usb.h"
#include <avr/interrupt.h>
#include <avr/io.h>
#include <avr/pgmspace.h>
#include <stdbool.h>
#include <util/delay.h>

int layout_num = 0;
bool has_unsent_packets = false;

bool state_layer[NUM_ROWS][NUM_COLS] = {
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}};  // Key States to do state changes

void matrix_init() {
  PORTD = 0; // push nothing out of port 0 to start with...
  DDRD &= ~(1 << PD1); // set PD1 to be input
  PORTD |= (1 << PD1) | (1 << PD0) | (1 << PD4); // set PD1 to be pull-up
}

void do_layer_led() { PORTC = layout_num << 6; }

void turn_on_leds() {
  PORTD &= ~(1 << PD5);
  PORTB &= ~(1 << PB0);
}

void turn_off_leds() {
  PORTD |= (1 << PD5);
  PORTB |= (1 << PB0);
}

void do_matrix_scan() {
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
  has_unsent_packets = true;
}

ISR(TIMER0_COMPA_vect) { // The USB packets are sent on a timer interrupt because OSX was showing strange USB keypress rejection errors due to high USB packet frequency
  if (has_unsent_packets) {
    usb_send();
		has_unsent_packets = false;
  }
}

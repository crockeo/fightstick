#pragma once

#include <stdbool.h>
#include <stdlib.h>

// Pin format, bits:
//   abcd_efgh
//
// ab = reserved
// cde = port number
// fgh = pin number
//
// ports:
// - 000 = B
// - 001 = C
// - 010 = D
// - 011 = E
// - 100 = F
//
// pins:
// - one's complement encoding
// - e.g. 111 = 7
typedef uint8_t pin_t;

// Based on pin numbering from ProMicro schematic.
// Ordering = counter-clockwise.
// https://cdn.sparkfun.com/assets/f/d/8/0/d/ProMicro16MHzv2.pdf
#define PIN_D1 0b010011
#define PIN_D0 0b010010
#define PIN_D2 0b010001
#define PIN_D3 0b010000
#define PIN_D4 0b010100
#define PIN_D5 0b001110
#define PIN_D6 0b010111
#define PIN_D7 0b011110
#define PIN_D8 0b000100
#define PIN_D9 0b000101
#define PIN_D10 0b000110
#define PIN_D16 0b000010
#define PIN_D14 0b000011
#define PIN_D15 0b000001
#define PIN_A0 0b100111
#define PIN_A1 0b100011
#define PIN_A2 0b100101
#define PIN_A3 0b100100

// TODO: how to make these functions not cost so many cycles?
void set_pull_up(pin_t pin) {
    uint8_t port = ((pin & 0b111000) >> 3) & 0b111;
    uint8_t raw_pin = pin & 0b111;

    if (port == 0b000) {
	DDRB &= ~(1 << raw_pin);
	PORTB |= (1 << raw_pin);
    } else if (port == 0b001) {
	DDRC &= ~(1 << raw_pin);
	PORTC |= (1 << raw_pin);
    } else if (port == 0b010) {
	DDRD &= ~(1 << raw_pin);
	PORTD |= (1 << raw_pin);
    } else if (port == 0b011) {
	DDRE &= ~(1 << raw_pin);
	PORTE |= (1 << raw_pin);
    }
}

bool is_pin_low(pin_t pin) {
    uint8_t port = ((pin & 0b111000) >> 3) & 0b111;
    uint8_t raw_pin = pin & 0b111;
    if (port == 0b000) {
	return (PINB & (1 << raw_pin)) == 0;
    } else if (port == 0b001) {
	return (PINC & (1 << raw_pin)) == 0;
    } else if (port == 0b010) {
	return (PIND & (1 << raw_pin)) == 0;
    } else if (port == 0b011) {
	return (PINE & (1 << raw_pin)) == 0;
    }
    return false;
}

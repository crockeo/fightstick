deploy: target/atmega32u4/release/fightstick.elf
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:target/atmega32u4/release/fightstick.elf

.PHONY :target/atmega32u4/release/fightstick.elf
target/atmega32u4/release/fightstick.elf:
	cargo build --release

deploy: build
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:target/atmega32u4/release/fightstick.elf

.PHONY: build
build:
	cargo +nightly-2022-07-17-aarch64-apple-darwin build --release

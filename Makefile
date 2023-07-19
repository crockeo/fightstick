FIRMWARE="target/atmega32u4/release/fightstick.elf"

C_SOURCES=$(shell find . -type f -name '*.c')

.PHONY: deploy
deploy: firmware
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:$(FIRMWARE)

.PHONY: firmware
firmware:
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: test
test:
	cargo test --release

.PHONY: doc
doc:
	cargo doc --open --release

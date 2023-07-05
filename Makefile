deploy: build
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:target/atmega32u4/release/fightstick.elf

.PHONY: build
build:
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: test
test:
	cargo test --release


.PHONY: doc
doc:
	cargo doc --open --release

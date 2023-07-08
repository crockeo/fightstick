C_SOURCES=$(shell find . -type f -name '*.c')

deploy: build-c
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:target/atmega32u4/release/fightstick.elf

.PHONY: build
build:
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: build-c
build-c:
	mkdir -p target/
	avr-gcc -Wall -Werror -O3 -mmcu=atmega32u4 -o target/atmega32u4/release/fightstick.elf $(C_SOURCES)

.PHONY: test
test:
	cargo test --release


.PHONY: doc
doc:
	cargo doc --open --release

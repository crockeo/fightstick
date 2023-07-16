C_SOURCES=$(shell find . -type f -name '*.c')
ELF_FILE="target/atmega32u4/release/fightstick.elf"

deploy: rust-build
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:$(ELF_FILE)

simulate: build
	simavr --mcu atmega32u4 $(ELF_FILE)

.PHONY: build
build:
	mkdir -p target/
	avr-gcc -Wall -Werror -O3 -mmcu=atmega32u4 -o $(ELF_FILE) $(C_SOURCES)

.PHONY: rust-build
rust-build:
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: rust-test
rust-test:
	cargo test --release


.PHONY: rust-doc
rust-doc:
	cargo doc --open --release

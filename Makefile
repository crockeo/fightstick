C_SOURCES=$(shell find . -type f -name '*.c')
ELF_FILE="target/atmega32u4/release/fightstick.elf"

deploy: build-c
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:$(ELF_FILE)

simulate: build-c
	simavr --mcu atmega32u4 $(ELF_FILE)

.PHONY: build
build:
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: build-c
build-c:
	mkdir -p target/
	avr-gcc -Wall -Werror -O3 -mmcu=atmega32u4 -o $(ELF_FILE) $(C_SOURCES)

.PHONY: test
test:
	cargo test --release


.PHONY: doc
doc:
	cargo doc --open --release

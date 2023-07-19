FIRMWARE="target/atmega32u4/release/fightstick.elf"
OBJ_DIR="target/atmega32u4/release/deps"

SOURCE_FILES=$(shell find . -type f -name '*.c' | sed 's|^\./||')
OBJ_NAMES=$(patsubst %.c,%.o,$(SOURCE_FILES))
OBJ_FILES=$(patsubst %,$(OBJ_DIR)/%,$(OBJ_NAMES))

LIBUSB=$(OBJ_DIR)/libusb.a

.PHONY: deploy
deploy: firmware
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:$(FIRMWARE)

.PHONY:
firmware: $(LIBUSB)
	echo $(OBJ_DIR)
	cargo build -Zbuild-std=core --target=./atmega32u4.json --release

.PHONY: test
test: $(OBJ_FILES)
	cargo test --release

.PHONY: $(LIBUSB)
$(LIBUSB): $(OBJ_FILES)
	avr-ar rcs $@ $^

.PHONY: $(OBJ_DIR)/%.o
$(OBJ_DIR)/%.o: %.c
	@mkdir -p $(OBJ_DIR)
	avr-gcc -Wall -Werror -O3 -mmcu=atmega32u4 -o $@ -c $<

.PHONY: doc
doc:
	cargo doc --open --release

.PHONY: clean
clean:
	rm -rf target
	cargo clean

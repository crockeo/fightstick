FIRMWARE="target/fightstick.elf"
SIMULATOR="target/simulator"

C_SOURCES=$(shell find . -type f -name '*.c' | grep -v simulator.c)

.PHONY: deploy
deploy: firmware
	avrdude \
		-p atmega32u4 \
		-c avr109 \
		-P /dev/tty.usbmodem11201 \
		-U flash:w:$(FIRMWARE)

.PHONY: simulate
simulate: simulator firmware
	$(SIMULATOR) --freq 16000000 --tracer --mcu atmega32u4 $(FIRMWARE)

.PHONY: simulator
simulator:
	mkdir -p $(shell dirname $(SIMULATOR))
	gcc -I/opt/homebrew/include -I/opt/homebrew/include/simavr -I/opt/homebrew/include/simavr/parts -L/opt/homebrew/lib -lsimavr -lelf -Wall -Werror -O3 -o $(SIMULATOR) simulator.c

.PHONY: firmware
firmware:
	mkdir -p $(shell dirname $(FIRMWARE))
	avr-gcc -Wall -Werror -O3 -mmcu=atmega32u4 -o $(FIRMWARE) $(C_SOURCES)

#include <libgen.h>
#include <pthread.h>
#include <simavr/avr_twi.h>
#include <simavr/parts/i2c_eeprom.h>
#include <simavr/sim_avr.h>
#include <simavr/sim_elf.h>
#include <simavr/sim_gdb.h>
#include <simavr/sim_vcd_file.h>
#include <stdio.h>
#include <stdlib.h>

avr_t* avr = NULL;
avr_vcd_t vcd_file;
i2c_eeprom_t eeprom;

int main(int argc, char *argv[]) {
    elf_firmware_t firmware;
    if (elf_read_firmware("target/fightstick.elf", &firmware) < 0) {
	fprintf(stderr, "Failed to run elf_read_firmware");
	return 1;
    }

    avr = avr_make_mcu_by_name(firmware.mmcu);
    avr_init(avr);
    avr_load_firmware(avr, &firmware);

    int state = cpu_Running;
    while (state != cpu_Done && state != cpu_Crashed) {
	state = avr_run(avr);
    }

    return 0;
}

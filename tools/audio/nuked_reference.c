/* Optional host-only black-box comparison with pinned unmodified Nuked-OPL3.
 * Unmodified LGPL header/core and source pins: tools/audio/reference/.
 * Input: LE rate:u32, frames:u32, event_count:u32; events(frame:u32,reg:u8,val:u8).
 * Output: mono left signed i16 LE. No device or original timing emulation.
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "opl3.h"
static uint32_t read32(void) {
    uint32_t value = 0;
    for (unsigned i = 0; i < 4; ++i) {
        int byte = getchar();
        if (byte == EOF) { fputs("truncated input\n", stderr); exit(1); }
        value |= (uint32_t)(unsigned char)byte << (8 * i);
    }
    return value;
}
int main(void) {
    uint32_t rate = read32(), frames = read32(), count = read32();
    if (rate < 8000 || rate > 192000 || frames > 16000000 || count > 1000000) return 1;
    uint32_t *times = calloc(count + 1, sizeof(uint32_t));
    unsigned char *registers = calloc(count + 1, 2);
    if (!times || !registers) return 1;
    for (uint32_t i = 0; i < count; ++i) {
        times[i] = read32();
        int reg = getchar(), value = getchar();
        if (reg == EOF || value == EOF || times[i] >= frames || (i && times[i] < times[i-1])) return 1;
        registers[i*2] = (unsigned char)reg; registers[i*2+1] = (unsigned char)value;
    }
    if (getchar() != EOF) return 1;
    opl3_chip chip; OPL3_Reset(&chip, rate);
    uint32_t event = 0;
    for (uint32_t frame = 0; frame < frames; ++frame) {
        while (event < count && times[event] == frame) {
            OPL3_WriteReg(&chip, registers[event*2], registers[event*2+1]); ++event;
        }
        int16_t out[2]; OPL3_GenerateResampled(&chip, out);
        uint16_t mono = (uint16_t)out[0]; putchar(mono & 255); putchar(mono >> 8);
    }
    free(times); free(registers); return ferror(stdout) ? 1 : 0;
}

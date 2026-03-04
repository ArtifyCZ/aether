#include "keyboard_priv.h"

#include <stdbool.h>
#include <stdint.h>

void print(const char *str);

static bool g_is_shift_down = false;

static const char map_no_shift[128] = {
    [0x02] = '1', [0x03] = '2', [0x04] = '3', [0x05] = '4', [0x06] = '5', [0x07] = '6', [0x08] = '7', [0x09] = '8',
    [0x0A] = '9', [0x0B] = '0',
    [0x10] = 'q', [0x11] = 'w', [0x12] = 'e', [0x13] = 'r', [0x14] = 't', [0x15] = 'y', [0x16] = 'u', [0x17] = 'i',
    [0x18] = 'o', [0x19] = 'p',
    [0x1E] = 'a', [0x1F] = 's', [0x20] = 'd', [0x21] = 'f', [0x22] = 'g', [0x23] = 'h', [0x24] = 'j', [0x25] = 'k',
    [0x26] = 'l',
    [0x2C] = 'z', [0x2D] = 'x', [0x2E] = 'c', [0x2F] = 'v', [0x30] = 'b', [0x31] = 'n', [0x32] = 'm',
    [0x39] = ' ',
    [0x1C] = '\n',
    [0x0E] = '\b',
};

static const char map_shift[128] = {
    [0x02] = '!', [0x03] = '@', [0x04] = '#', [0x05] = '$', [0x06] = '%', [0x07] = '^', [0x08] = '&', [0x09] = '*',
    [0x0A] = '(', [0x0B] = ')',
    [0x10] = 'Q', [0x11] = 'W', [0x12] = 'E', [0x13] = 'R', [0x14] = 'T', [0x15] = 'Y', [0x16] = 'U', [0x17] = 'I',
    [0x18] = 'O', [0x19] = 'P',
    [0x1E] = 'A', [0x1F] = 'S', [0x20] = 'D', [0x21] = 'F', [0x22] = 'G', [0x23] = 'H', [0x24] = 'J', [0x25] = 'K',
    [0x26] = 'L',
    [0x2C] = 'Z', [0x2D] = 'X', [0x2E] = 'C', [0x2F] = 'V', [0x30] = 'B', [0x31] = 'N', [0x32] = 'M',
    [0x39] = ' ',
    [0x1C] = '\n',
    [0x0E] = '\b',
};


static inline bool is_break(uint8_t sc) { return (sc & 0x80) != 0; }

static inline uint8_t make_code(uint8_t sc) { return sc & 0x7F; }

bool keyboard_arch_parse_scancode(uint8_t sc, char *out) {
    if (!is_break(sc) && (sc == 0x2A || sc == 0x36)) {
        g_is_shift_down = true;
        return false;
    }
    if (is_break(sc) && (make_code(sc) == 0x2A || make_code(sc) == 0x36)) {
        g_is_shift_down = false;
        return false;
    }

    if (is_break(sc)) return false; // ignore releases for now

    uint8_t mc = make_code(sc);
    char c = g_is_shift_down ? map_shift[mc] : map_no_shift[mc];
    if (c == 0x0) {
        return false;
    }

    *out = c;
    return true;
}

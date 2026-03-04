//
// Created by artify on 2/4/26.
//

#include "terminal.h"

#include "psf.h"
#include "stddef.h"
#include "stdint.h"

static struct limine_framebuffer *framebuffer = NULL;

static uint32_t cursor_x = 0;
static uint32_t cursor_y = 0;

static uint32_t foreground_color = 0x00FFFFFF; // default full white
static uint32_t background_color = 0x00000000; // default full black

void terminal_init(struct limine_framebuffer *_framebuffer) {
    framebuffer = _framebuffer;
}

void terminal_set_foreground_color(uint32_t color) {
    foreground_color = color;
}

void terminal_set_background_color(uint32_t color) {
    background_color = color;
}

void terminal_clear(void) {
    for (uint32_t y = 0; y < framebuffer->height; y++) {
        for (uint32_t x = 0; x < framebuffer->width; x++) {
            volatile uint32_t *fb_ptr = framebuffer->address;
            fb_ptr += x + y * (framebuffer->pitch / 4);
            fb_ptr[0] = background_color;
        }
    }
}

void terminal_print_char(char c) {
    if (c == '\n') {
        cursor_x = 0;
        cursor_y++;
        return;
    }

    if (c == '\b') {
        if (cursor_x > 0) {
            cursor_x--;
        }
        psf_render_char(' ', cursor_x, cursor_y, foreground_color, background_color);
        return;
    }

    psf_render_char(c, cursor_x, cursor_y, foreground_color, background_color);
    cursor_x++;
}

void terminal_print(const char *message) {
    for (size_t i = 0; message[i] != 0x00; i++) {
        terminal_print_char(message[i]);
    }
}

void terminal_println(const char *message) {
    terminal_print(message);
    terminal_print_char('\n');
}

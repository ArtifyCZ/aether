//
// Created by artify on 2/4/26.
//

#ifndef KERNEL_2026_01_31_TERMINAL_H
#define KERNEL_2026_01_31_TERMINAL_H

#include <limine.h>

void terminal_init(struct limine_framebuffer *framebuffer);

void terminal_set_foreground_color(uint32_t color);

void terminal_set_background_color(uint32_t color);

void terminal_clear(void);

void terminal_print_char(char c);

void terminal_print(const char *message);

void terminal_println(const char *message);

#endif //KERNEL_2026_01_31_TERMINAL_H

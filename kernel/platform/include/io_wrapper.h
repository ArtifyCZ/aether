//
// Created by Richard Tichý on 05.02.2026.
//

#ifndef KERNEL_IO_WRAPPER_H
#define KERNEL_IO_WRAPPER_H

#include <stdint.h>

uint8_t inb(uint16_t port);

void outb(uint16_t port, uint8_t val);

#endif //KERNEL_IO_WRAPPER_H

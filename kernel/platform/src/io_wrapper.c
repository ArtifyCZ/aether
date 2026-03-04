//
// Created by Richard Tichý on 05.02.2026.
//

#include "io_wrapper.h"


uint8_t inb(uint16_t port) {
    uint8_t ret = 0;
#if defined (__x86_64__)
    __asm__ volatile ( "inb %w1, %b0"
        : "=a"(ret)
        : "Nd"(port)
        : "memory");
#else
#warning "Not implemented yet"
#endif
    return ret;
}

void outb(uint16_t port, uint8_t val) {
#if defined (__x86_64__)
    __asm__ volatile ( "outb %b0, %w1" : : "a"(val), "Nd"(port) : "memory");
    /* There's an outb %al, $imm8 encoding, for compile-time constant port numbers that fit in 8b. (N constraint).
     * Wider immediate constants would be truncated at assemble-time (e.g. "i" constraint).
     * The  outb  %al, %dx  encoding is the only option for all other cases.
     * %1 expands to %dx because  port  is a uint16_t.  %w1 could be used if we had the port number a wider C type */
#else
#warning "Not implemented yet"
#endif
}

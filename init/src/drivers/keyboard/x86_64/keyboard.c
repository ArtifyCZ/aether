#include "drivers/keyboard/keyboard.h"

#include "keyboard_priv.h"
#include "stdbool.h"
#include "stddef.h"
#include "stdint.h"
#include "../../../io.x86_64.h"
#include "../../../syscalls.h"

#define KEYBOARD_COMMAND_PORT 0x64
#define KEYBOARD_DATA_PORT 0x60

#define KEYBOARD_IRQ 0x01

static bool keyboard_read_scancode(uint8_t *out);

static __attribute__((noreturn)) void keyboard_thread(void);

void print(const char *message);

void keyboard_init(void) {
    const uintptr_t stack_size = 0x4000;
    uintptr_t stack_base;
    if (sys_mmap(0x7FFFFFF54000, stack_size, 0, 0, &stack_base)) {
        print("FAILED TO ALLOCATE STACK!");
        sys_exit();
    }
    const uintptr_t stack_top = stack_base + stack_size;
    if (sys_clone(0, (void *) stack_top, keyboard_thread, NULL)) {
        print("FAILED TO CLONE THREAD!");
        sys_exit();
    }
}

static __attribute__((noreturn)) void keyboard_thread(void) {
    uint8_t sc;
    while (keyboard_read_scancode(&sc)) {
        // clear keyboard buffer
    }

    while (1) {
        if (sys_irq_unmask(KEYBOARD_IRQ)) {
            print("FAILED TO UNMASK KEYBOARD IRQ!");
            sys_exit();
        }

        if (sys_irq_wait(KEYBOARD_IRQ)) {
            print("FAILED TO WAIT FOR KEYBOARD IRQ!");
            sys_exit();
        }

        char c;
        while (keyboard_read_scancode(&sc)) {
            if (keyboard_arch_parse_scancode(sc, &c)) {
                const char message[2] = {c, '\0'};
                print(message);
            }
        }
    }
}

static bool keyboard_read_scancode(uint8_t *out) {
    const uint8_t status = inb(KEYBOARD_COMMAND_PORT);
    if ((status & 0x01) == 0) {
        return false;
    }

    *out = inb(KEYBOARD_DATA_PORT);
    return true;
}

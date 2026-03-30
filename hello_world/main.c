#include "libs/libsyscall/syscalls.h"
#include "stddef.h"
#include <stdint.h>

void print(const char *message) {
    size_t length = 0;
    while (message[length] != '\0') {
        length++;
    }

    sys_write(1, message, length);
}


__attribute__((noreturn)) void _start(uint64_t *ipc_base) {
    print("Hello world from initrd-loaded program!");
    if (ipc_base == NULL) {
        print("No IPC base provided!\n");
        sys_exit();
    }
    print("IPC base provided, attempting a write...");
    ipc_base[0] = 1;
    print("Write successful, exiting...\n");
    sys_exit();
    while (1);
}

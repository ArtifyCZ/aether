#include "libs/libsyscall/syscalls.h"
#include "kernel/api/init/boot_info.h"
#include <stdint.h>

int main(struct boot_info *boot_info);

static void print(const char *message) {
    size_t length = 0;
    while (message[length] != '\0') {
        length++;
    }

    sys_write(1, message, length);
}

__attribute__((noreturn)) void _start(struct boot_info *boot_info) {
    sys_write(1, "1", 1);
    sys_write(1, "2", 1);
    print("Hello from _start!\n");
    print("Hello from _start again!\n");
    int exit_code = main(boot_info);
    print("Exiting...\n");
    sys_exit(); // @TODO: also pass the exit code
    print("SHOULD NOT HAPPEN! init/src/entry.c");
    while (1) {
    }
}

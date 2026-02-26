#include "syscalls.h"

int main(void);

static void print(const char *message) {
    size_t length = 0;
    while (message[length] != '\0') {
        length++;
    }

    sys_write(1, message, length);
}

__attribute__((noreturn)) void _start(void) {
    sys_write(1, "1", 1);
    sys_write(1, "2", 1);
    print("Hello from _start!\n");
    print("Hello from _start again!\n");
    int exit_code = main();
    print("Exiting...\n");
    sys_exit(); // @TODO: also pass the exit code
    while (1) {
    }
}

#include "init/include/tar.h"
#include "kernel/api/init/boot_info.h"
#include "libs/libsyscall/syscalls.h"
#include "drivers/keyboard/keyboard.h"
#include "drivers/serial/serial.h"
#include <stddef.h>
#include <stdint.h>

void print(const char *message) {
    size_t length = 0;
    while (message[length] != '\0') {
        length++;
    }

    // sys_write(1, message, length);
    serial_print(message);
}

void rest(void) {
    int j = 0;

    while (1) {
        for (volatile int i = 0; i < 10000000; i++);
        const char number[] = {j + '0', '\n', 0x00};
        print(number);
        j++;
        if (j == 10) {
            break;
        }
    }
}

__attribute__ ((noreturn)) void second_thread(void) {
    print("Hello from second thread!\n");
    rest();
    sys_exit();
    print("THIS SHOULD NOT HAPPEN!");
    while (1) {}
}

int main(struct boot_info *boot_info) {
    if (serial_init()) {
        const char message[] = "Failed to initialize serial port!\n";
        sys_write(1, message, sizeof(message));
        while(1);
    }

    const char message[] = "Hello world from user-space!\n";
    print(message);

    print("Trying to invoke clone syscall...\n");
    uintptr_t stack_base;
    if (sys_mmap(0x7FFFFFFF8000, 0x4000, 0, 0, &stack_base)) {
        print("Failed to allocate stack!\n");
        while(1);
    }
    const void *stack_top = (void *) (stack_base + 0x4000);
    if (stack_base < 0x400000) { // If it returned something tiny like 3, it's an error!
        print("mmap failed or returned invalid address!\n");
        while(1);
    }

    sys_clone(0, stack_top, second_thread, NULL);
    print("Parent is moving on...\n");
    keyboard_init();
    print("Keyboard initialized!\n");

    print("Trying to parse initrd...\n");
    char *initrd_message;
    size_t initrd_message_size;
    tar_find_file(boot_info->initrd_start, boot_info->initrd_size, "initrd_message.txt", &initrd_message, &initrd_message_size);
    if (initrd_message) {
        print("Found initrd_message.txt in initrd! Contents:\n");
        print(initrd_message);
        print("\n");
    } else {
        print("Failed to find initrd_message.txt in initrd!\n");
    }
    print("Finished parsing initrd!\n");

    return 0;
}

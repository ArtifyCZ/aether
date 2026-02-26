#include "syscalls.h"

void print(const char *message) {
    size_t length = 0;
    while (message[length] != '\0') {
        length++;
    }

    sys_write(1, message, length);
}

void rest(void) {
    int j = 0;

    while (1) {
        for (volatile int i = 0; i < 10000000; i++);
        char number = j + '0';
        sys_write(1, &number, 1);
        char newline = '\n';
        sys_write(1, &newline, 1);
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

void *alloc_stack(void) {
    const uint64_t length = 0x4000;
    const uint64_t second_thread_stack = sys_mmap(0x7FFFFFFF8000, length, 0, 0);
    if (second_thread_stack == 0) {
        print("Failed to allocate stack!\n");
        while (1) {}
    }
    const uint64_t stack_top = second_thread_stack + length;
    return (void *) stack_top;
}

int main(void) {
    const char message[] = "Hello world from user-space!\n";
    print(message);

    print("Trying to invoke clone syscall...\n");
    // @TODO: add stack allocation
    // const void *second_thread_stack = alloc_stack();
    const uint64_t stack_base = sys_mmap(0x7FFFFFFF8000, 0x4000, 0, 0);
    const void *stack_top = (void *) (stack_base + 0x4000);
    if (stack_base < 0x400000) { // If it returned something tiny like 3, it's an error!
        print("mmap failed or returned invalid address!\n");
        while(1);
    }

    sys_clone(0, stack_top - 0x10, second_thread);
    print("Parent is moving on...\n");

    rest();
    return 0;
}

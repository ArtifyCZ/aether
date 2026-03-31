#include "init/include/tar.h"
#include "kernel/api/init/boot_info.h"
#include "kernel/api/syscalls/syscall_list.h"
#include "libs/libsyscall/syscalls.h"
#include "drivers/keyboard/keyboard.h"
#include "drivers/serial/serial.h"
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "elf.h"

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
    print("THIS SHOULD NOT HAPPEN!\n");
    while (1) {
    }
}

void spawn_hello_world(const uint64_t proc_handle, const uintptr_t entrypoint_addr) {
    print("Allocating stack...\n");
    uintptr_t stack_base = 0x7FFFFFFF8000;
    const size_t stack_size = 4 * 0x1000;
    if (sys_proc_mmap(proc_handle, stack_base, stack_size, SYS_PROT_READ | SYS_PROT_WRITE, 0, &stack_base)) {
        print("Failed to allocate stack!\n");
        return;
    }
    const uintptr_t stack_top = stack_base + stack_size;

    print("Allocating IPC area...\n");
    uintptr_t ipc_base = 0x7FFFFFF00000;
    const size_t ipc_size = 0x1000;
    void *ipc_ptr = NULL;
    if (sys_proc_mmap(proc_handle, ipc_base, ipc_size, SYS_PROT_READ | SYS_PROT_WRITE, SYS_MMAP_FL_MIRROR, (uintptr_t *) &ipc_ptr)) {
        print("Failed to allocate IPC buffer!\n");
        return;
    }
    memset(ipc_ptr, 0, ipc_size);

    print("Spawning hello_world...\n");
    uint64_t task_id;
    if (sys_proc_spawn(proc_handle, 0, stack_top, entrypoint_addr, ipc_base, &task_id)) {
        print("Failed to spawn hello_world...\n");
    } else {
        print("Spawned hello_world successfully!\n");
    }
}

int main(struct boot_info *boot_info) {
    if (serial_init()) {
        const char message[] = "Failed to initialize serial port!\n";
        sys_write(1, message, sizeof(message));
        while (1);
    }

    const char message[] = "Hello world from user-space!\n";
    print(message);

    print("Trying to invoke clone syscall...\n");
    uintptr_t stack_base;
    if (sys_mmap(0x7FFFFFFF8000, 0x4000, 0, 0, &stack_base)) {
        print("Failed to allocate stack!\n");
        while (1);
    }
    const void *stack_top = (void *) (stack_base + 0x4000);
    if (stack_base < 0x400000) {
        // If it returned something tiny like 3, it's an error!
        print("mmap failed or returned invalid address!\n");
        while (1);
    }

    sys_clone(0, stack_top, second_thread, NULL);
    print("Parent is moving on...\n");
    keyboard_init();
    print("Keyboard initialized!\n");

    print("Trying to parse initrd...\n");
    void *hello_world_elf;
    size_t hello_world_elf_size;
    tar_find_file(
        boot_info->initrd_start,
        boot_info->initrd_size,
        "bin/hello_world",
        &hello_world_elf,
        &hello_world_elf_size
    );
    if (hello_world_elf) {
        print("Found hello_world in initrd!\n");
        uintptr_t v_entrypoint_addr;
        uint64_t proc_handle;
        if (elf_load(hello_world_elf, &v_entrypoint_addr, &proc_handle)) {
            print("Failed to parse/load hello_world elf!\n");
            return -1;
        }
        print("Successfully loaded hello_world elf!\n");
        spawn_hello_world(proc_handle, v_entrypoint_addr);
    } else {
        print("Failed to find hello_world in initrd!\n");
    }
    print("Finished parsing initrd!\n");

    return 0;
}

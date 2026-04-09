#pragma once

#include "kernel/api/syscalls/syscall_gen.h"
#include <stdint.h>

#define GEN_ENUM(NAME, num, message) \
    SYS_##NAME = num,

enum syscall_err : uint64_t {
    SYSCALLS_ERR_LIST(GEN_ENUM)
};

typedef enum syscall_err sys_err_t;

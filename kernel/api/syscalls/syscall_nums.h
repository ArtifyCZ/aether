#pragma once

#include "syscall_list.h"
#include <stdint.h>

#define GEN_ENUM(name, NAME, num, ret, count, ...) \
SYS_##NAME = num,

enum syscall_num : uint64_t {
    SYSCALLS_LIST(GEN_ENUM)
};

typedef enum syscall_num syscall_num_t;

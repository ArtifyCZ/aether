#pragma once

#include "syscall_list.h"
#include "syscall_errs.h"

struct syscall_result {
    enum syscall_err err_code;
    uint64_t value;
};

typedef struct syscall_result sys_result_t;

/* --- Architectural Abstraction --- */
// Expecting the platform to provide:
// sys_result_t __arch_syscall(uint64_t num, uint64_t a1, uint64_t a2, ...);

/* --- Internal Helpers --- */
#define _SS_LIST0()              void
#define _SS_LIST1(t1, n1)        t1 n1
#define _SS_LIST2(t1, n1, ...)   t1 n1, _SS_LIST1(__VA_ARGS__)
#define _SS_LIST3(t1, n1, ...)   t1 n1, _SS_LIST2(__VA_ARGS__)
#define _SS_LIST4(t1, n1, ...)   t1 n1, _SS_LIST3(__VA_ARGS__)
#define _SS_LIST5(t1, n1, ...)   t1 n1, _SS_LIST4(__VA_ARGS__)

#define _SS_VAL0()               0
#define _SS_VAL1(t1, n1)         (uint64_t)(n1)
#define _SS_VAL2(t1, n1, ...)    (uint64_t)(n1), _SS_VAL1(__VA_ARGS__)
#define _SS_VAL3(t1, n1, ...)    (uint64_t)(n1), _SS_VAL2(__VA_ARGS__)
#define _SS_VAL4(t1, n1, ...)    (uint64_t)(n1), _SS_VAL3(__VA_ARGS__)
#define _SS_VAL5(t1, n1, ...)    (uint64_t)(n1), _SS_VAL4(__VA_ARGS__)

/* --- The Dual-Register Wrapper Generator --- */

// Logic: Every syscall returns sys_result_t.
// If userspace wants the raw value, they access .value
#define SYSCALL_USER_WRAPPER(name, NAME, num, ret, count, ...) \
    _SS_USER_WRAPPER_##ret(name, NAME, num, ret, count, __VA_ARGS__) \
    /* */

#define _SS_USER_WRAPPER_void(name, NAME, num, ret, count, ...) \
    static inline sys_err_t sys_##name(_SS_LIST##count(__VA_ARGS__)) { \
        return __arch_syscall(num, _SS_VAL##count(__VA_ARGS__)).err_code; \
    } \
    /* */

#define _SS_USER_WRAPPER_uint64_t(name, NAME, num, ret, count, ...) \
    static inline sys_err_t sys_##name(_SS_LIST##count(__VA_ARGS__), ret *out) { \
        sys_result_t result = __arch_syscall(num, _SS_VAL##count(__VA_ARGS__)); \
        if (result.err_code == SYS_SUCCESS && out != NULL) *out = result.value; \
        return result.err_code; \
    } \
    /* */

#define _SS_USER_WRAPPER_uintptr_t(name, NAME, num, ret, count, ...) \
    static inline sys_err_t sys_##name(_SS_LIST##count(__VA_ARGS__), ret *out) { \
        sys_result_t result = __arch_syscall(num, _SS_VAL##count(__VA_ARGS__)); \
        if (result.err_code == SYS_SUCCESS && out != NULL) *out = result.value; \
        return result.err_code; \
    } \
    /* */

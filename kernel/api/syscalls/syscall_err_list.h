#pragma once

// Every syscall returns an error code (if it is to switch back to the task),
// and optionally a return value. Both are done through registers. Using the
// return value if error code != SYS_SUCCESS is undefined behavior.
// Error code values are guaranteed to be unique, even in cases of error codes
// specific to a single syscall.

#define SYSCALLS_ERR_LIST(X) \
    /* NAME,        Num,        Message */ \
    X(SUCCESS,      0x00000,    "Success") \
    X(EFAULT,       0x00001,    "Bad address") \
    X(EBADF,        0x00002,    "Bad file descriptor") \
    X(EINVAL,       0x00003,    "Invalid argument") \
    X(ENOSYS,       0x00004,    "Function not implemented") \
    /* End of the list */

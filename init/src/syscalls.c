#include "syscalls.h"

#define STRINGIFY2(X) #X
#define STRINGIFY(X) STRINGIFY2(X)

#define SYS_ERR_MSG(NAME, num, message) \
    case num: \
        return "(" STRINGIFY(num) ") " STRINGIFY(NAME) ": " message; \
    /* */

const char *sys_err_get_message(sys_err_t err_code) {
    switch (err_code) {
        SYSCALLS_ERR_LIST(SYS_ERR_MSG)
        default:
            // @TODO: also add the err code value to the returned message
            return "(unknown error)";
    }
}

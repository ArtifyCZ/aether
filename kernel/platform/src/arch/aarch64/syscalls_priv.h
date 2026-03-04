#pragma once

#include "cpu_interrupts.h"
#include <stdbool.h>

bool syscalls_interrupt_handler(struct interrupt_frame **frame);

#pragma once

#include "interrupts.h"
#include <stdint.h>
#include <stdbool.h>

// Callback signature for a tick handler.
// Returns true for the tick to be acknowledged.
typedef bool (*timer_tick_handler_t)(struct interrupt_frame **frame, void *priv);

void timer_init(uint32_t freqHz);

/**
 * @param handler Callback handler to be called for each tick
 * @param priv Private arguments to be passed to the callback
 */
void timer_set_tick_handler(timer_tick_handler_t handler, void *priv);

uint64_t timer_get_ticks(void);

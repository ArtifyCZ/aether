#include "timer.h"

#include <stddef.h>

#include "gic.h"
#include "interrupts.h"

#define IRQ_INTERRUPT_VECTOR 0x1B

static volatile uint32_t g_freqHz;

static volatile uint64_t g_ticks = 0;

static volatile timer_tick_handler_t g_tick_handler;

static volatile void *g_tick_handler_priv;

bool timer_irq_handler(struct interrupt_frame **frame, void *priv);

void timer_init(uint32_t freqHz) {
    if (freqHz == 0)
        freqHz = 100;

    g_freqHz = freqHz;
    g_ticks = 0;
    g_tick_handler = NULL;
    g_tick_handler_priv = NULL;

    uint64_t freq;
    // 1. Get the system counter's frequency (usually 62.5MHz on QEMU)
    __asm__ volatile("mrs %0, cntfrq_el0" : "=r"(freq));

    uint32_t ticks_per_int = freq / freqHz;

    // 2. Set the timer value (countdown)
    __asm__ volatile("msr cntv_tval_el0, %0" : : "r"((uint64_t) ticks_per_int));

    // 3. Enable the timer and unmask the interrupt
    // Control register: bit 0 = enable, bit 1 = imask (0 to unmask)
    __asm__ volatile("msr cntv_ctl_el0, %0" : : "r"((uint64_t) 1));

    // 4. Register the handler in your common system
    // 0x1B is the standard Virtual Timer PPI on QEMU virt
    gic_configure_interrupt(IRQ_INTERRUPT_VECTOR, 0x80);
    interrupts_register_handler(IRQ_INTERRUPT_VECTOR, timer_irq_handler, NULL);
}

void timer_set_tick_handler(timer_tick_handler_t handler, void *priv) {
    g_tick_handler = handler;
    g_tick_handler_priv = priv;
}

uint64_t timer_get_ticks(void) {
    return g_ticks;
}

bool timer_irq_handler(struct interrupt_frame **frame, void *priv) {
    // Reset the timer for the next tick! (Crucial: ARM timers aren't periodic by default)
    uint64_t freq;
    __asm__ volatile("mrs %0, cntfrq_el0" : "=r"(freq));
    __asm__ volatile("msr cntv_tval_el0, %0" : : "r"(freq / g_freqHz));

    g_ticks++;

    if (g_tick_handler != NULL) {
        return g_tick_handler(frame, (void *) g_tick_handler_priv);
    }

    return true;
}

#include "timer.h"

#include "early_console.h"
#include "interrupts.h"
#include "io_wrapper.h"
#include "stddef.h"
#include "lapic.h"

#define PIT_FREQ 1193182
#define TARGET_MS 10

#define PIT_CH0_DATA  0x40
#define PIT_CMD       0x43

#define LAPIC_TIMER_VECTOR 0x30

// Value 0x3 corresponds to a divisor of 16
#define LAPIC_TIMER_DIVISOR 0x3

bool lapic_timer_handler(struct interrupt_frame **frame, void *priv);

static volatile uint32_t g_lapic_ticks_per_ms = 0;
static volatile uint64_t g_ticks = 0;
static volatile timer_tick_handler_t g_tick_handler = NULL;
static volatile void *g_tick_handler_priv = NULL;

static void lapic_calibrate_timer(void) {
    // 1. Software-enable LAPIC via Spurious Vector Register
    uint32_t svr = lapic_read(LAPIC_REG_SVR);
    lapic_write(LAPIC_REG_SVR, svr | 0x100);

    // 2. Set divider
    lapic_write(LAPIC_REG_TDCR, LAPIC_TIMER_DIVISOR);

    // 3. Prepare PIT for 10ms delay (Mode 0)
    uint16_t pit_reload_value = PIT_FREQ / 100;
    outb(PIT_CMD, 0x30);
    outb(PIT_CH0_DATA, (uint8_t)(pit_reload_value & 0xFF));
    outb(PIT_CH0_DATA, (uint8_t)((pit_reload_value >> 8) & 0xFF));

    // 4. Start LAPIC timer countdown from max
    lapic_write(LAPIC_REG_TICRET, 0xFFFFFFFF);

    // 5. Poll PIT until it hits zero
    while (1) {
        outb(PIT_CMD, 0x00); // Latch
        uint8_t low = inb(PIT_CH0_DATA);
        uint8_t high = inb(PIT_CH0_DATA);
        if ((low | (high << 8)) == 0) break;
    }

    // 6. Calculate how many ticks passed in 10ms
    uint32_t current_count = lapic_read(LAPIC_REG_TCCR);
    uint32_t ticks_in_10ms = 0xFFFFFFFF - current_count;

    // Stop the timer
    lapic_write(LAPIC_REG_TICRET, 0);

    g_lapic_ticks_per_ms = ticks_in_10ms / TARGET_MS;
}

void timer_init(uint32_t freqHz) {
    if (freqHz == 0) freqHz = 100;

    // Initialize LAPIC abstraction (Maps MMIO if not already done)
    lapic_init(0xFEE00000);

    // Mask legacy PIC
    outb(0x21, 0xFF);
    outb(0xA1, 0xFF);

    lapic_calibrate_timer();

    // Register the IDT handler
    interrupts_register_handler(LAPIC_TIMER_VECTOR, lapic_timer_handler, NULL);

    // Calculate reload value based on calibrated frequency
    uint32_t initial_count = (g_lapic_ticks_per_ms * 1000) / freqHz;

    // Configure LVT Timer: Periodic Mode (bit 17) + Vector
    lapic_write(LAPIC_REG_LVT_TMR, LAPIC_TIMER_VECTOR | (1 << 17));
    lapic_write(LAPIC_REG_TDCR, LAPIC_TIMER_DIVISOR);
    lapic_write(LAPIC_REG_TICRET, initial_count);

    early_console_print("LAPIC Timer: initialized at ");
    early_console_print_hex_u64(freqHz);
    early_console_println(" Hz");
}

void timer_set_tick_handler(timer_tick_handler_t handler, void *priv) {
    g_tick_handler = handler;
    g_tick_handler_priv = priv;
}

uint64_t timer_get_ticks(void) {
    return g_ticks;
}

bool lapic_timer_handler(struct interrupt_frame **frame, void *priv) {
    g_ticks++;

    if (g_tick_handler != NULL) {
        return g_tick_handler(frame, (void *)g_tick_handler_priv);
    }

    return true;
}

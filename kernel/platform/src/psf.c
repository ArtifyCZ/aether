#include "psf.h"

#include "early_console.h"
#include "stdbool.h"
#include "stddef.h"

static struct PSF2_Font font = {};
static void *font_data = NULL;
static uint64_t font_size = 0;

static struct limine_framebuffer *framebuffer = NULL;

static inline bool psf_is_psf1(void *font_address) {
    return ((uint16_t *) font_address)[0] == PSF1_FONT_MAGIC;
}

static void psf_init_psf1(const struct PSF1_Header *font_header, void *glyphs, uint64_t font_data_size) {
    uint32_t numglyph = 0;
    if (font_header->fontMode & PSF1_MODE512) {
        numglyph = 512;
    } else {
        numglyph = 256;
    }
    font.magic = PSF_FONT_MAGIC;
    font.version = 0;
    font.headersize = sizeof(struct PSF1_Header);
    font.flags = font_header->fontMode;
    font.numglyph = numglyph;
    font.bytesperglyph = font_header->characterSize;
    font.height = font_header->characterSize;
    font.width = 8;
    font_data = glyphs;
    font_size = font_data_size;
}

void psf_init(void *font_address, uint64_t font_size_arg, struct limine_framebuffer *framebuffer_arg) {
    early_console_println("Initializing PC Screen Font...");

    if (font_address == NULL) {
        early_console_println("Got NULL ptr as font address; cannot initialize PSF");
        return;
    }

    if (((uint16_t *) font_address)[0] != PSF1_FONT_MAGIC) {
        early_console_println("Got invalid font magic; cannot initialize PSF");
        return;
    }

    psf_init_psf1(font_address, (void *) ((uintptr_t) font_address + sizeof(struct PSF1_Header)),
                  font_size_arg - sizeof(struct PSF1_Header));

    framebuffer = framebuffer_arg;

    early_console_println("");
}

void psf_render_char(char c, uint32_t col, uint32_t row, uint32_t foreground_color, uint32_t background_color) {
    uint8_t *glyph = &font_data[c * font.bytesperglyph];
    uint32_t stride = font.bytesperglyph / font.height;
    uint32_t y0;
    uint32_t x0;

    for (y0 = 0; y0 < font.height; ++y0) {
        for (x0 = 0; x0 < font.width; ++x0) {
            uint8_t bits = glyph[y0 * stride + x0 / 8];
            uint8_t bit = bits >> (7 - x0 % 8) & 1;
            volatile uint32_t *fb_ptr = framebuffer->address;
            fb_ptr += (col * font.width + x0) + (row * font.height + y0) * (framebuffer->pitch / 4);
            if (bit) fb_ptr[0] = foreground_color;
            else fb_ptr[0] = background_color;
        }
    }
}

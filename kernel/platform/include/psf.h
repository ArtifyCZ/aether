//
// Created by artify on 2/4/26.
//

#ifndef KERNEL_2026_01_31_PSF_H
#define KERNEL_2026_01_31_PSF_H

#include <stdint.h>
#include <limine.h>


#define PSF1_FONT_MAGIC 0x0436
#define PSF1_MODE512    0x01

/*
    For PSF1 glyph width is always = 8 bits
    and glyph height = characterSize
*/
struct PSF1_Header {
    uint16_t magic; // Magic bytes for identification.
    uint8_t fontMode; // PSF font mode.
    uint8_t characterSize; // PSF character size.
};


#define PSF_FONT_MAGIC 0x864ab572

/*
    PSF2
*/
struct PSF2_Font {
    uint32_t magic;         /* magic bytes to identify PSF */
    uint32_t version;       /* zero */
    uint32_t headersize;    /* offset of bitmaps in file, 32 */
    uint32_t flags;         /* 0 if there's no unicode table */
    uint32_t numglyph;      /* number of glyphs */
    uint32_t bytesperglyph; /* size of each glyph */
    uint32_t height;        /* height in pixels */
    uint32_t width;         /* width in pixels */
};

void psf_init(void *font_address, uint64_t font_size, struct limine_framebuffer *framebuffer);

void psf_render_char(char c, uint32_t col, uint32_t row, uint32_t foreground_color, uint32_t background_color);

#endif //KERNEL_2026_01_31_PSF_H

#include "init/include/tar.h"

#include <stddef.h>
#include <stdint.h>

void print(const char *message);

typedef struct {
    char name[100];     // Filename
    char mode[8];
    char uid[8];
    char gid[8];
    char size[12];      // Size in octal ASCII!!
    char mtime[12];
    char checksum[8];
    char typeflag;
    // ... total 512 bytes
} __attribute__((packed)) tar_header_t;

// Helper to convert octal string to integer
static inline size_t get_size(const char *in) {
    size_t size = 0;
    for (int i = 0; i < 11; i++) {
        size = size * 8 + (in[i] - '0');
    }
    return size;
}

static inline int strcmp(const char *a, const char *b) {
    while (*a && *b && *a == *b) {
        a++;
        b++;
    }
    return (unsigned char)*a - (unsigned char)*b;
}

void tar_find_file(void *tar_addr, size_t tar_size, const char *filename, void **file_data, size_t *file_size) {
    uint8_t *ptr = tar_addr;
    
    while (ptr < (uint8_t *)(tar_addr + tar_size)) {
        tar_header_t *header = (tar_header_t *)ptr;

        if (header->name[0] == '\0') break; // End of archive

        size_t current_file_size = get_size(header->size);
        if (strcmp(header->name, filename) == 0) {
            *file_data = ptr + 512; // Data is immediately after the 512-byte header
            *file_size = current_file_size;
            return;
        }

        // Move to the next header: 
        // 512 (header) + file_size (aligned to 512 bytes)
        ptr += 512 + ((current_file_size + 511) & ~511);
    }

    *file_data = NULL; // Not found
    *file_size = 0;
}

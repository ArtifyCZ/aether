#pragma once

#include <stddef.h>
#include <stdint.h>

/**
 * Searches for a file in a tar archive loaded in memory.
 * @param tar_addr Pointer to the start of the tar archive in memory.
 * @param tar_size Size of the tar archive in bytes.
 * @param filename Null-terminated string of the filename to search for.
 * @param file_data Output pointer that will point to the file's data if found.
 * @param file_size Output variable that will hold the size of the file if found.
 * If the file is not found, *file_data will be set to NULL and *file_size will be set to 0.
 */
void tar_find_file(void *tar_addr, size_t tar_size, const char *filename, void **file_data, size_t *file_size);

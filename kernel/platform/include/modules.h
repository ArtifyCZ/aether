//
// Created by artify on 2/4/26.
//

#ifndef KERNEL_2026_01_31_MODULES_H
#define KERNEL_2026_01_31_MODULES_H

#include <limine.h>

void modules_init(struct limine_module_response *modules);

const struct limine_file *module_find(const char *associated_string);

#endif //KERNEL_2026_01_31_MODULES_H

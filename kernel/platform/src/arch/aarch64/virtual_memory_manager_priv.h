#pragma once

#define DESC_VALID      (1ULL << 0)
#define DESC_TABLE      (1ULL << 1)
#define DESC_PAGE       (1ULL << 1)
#define DESC_USER       (1ULL << 6)
#define DESC_RO         (1ULL << 7)
#define DESC_AF         (1ULL << 10)
#define DESC_SH_INNER   (3ULL << 8)
#define DESC_PXN        (1ULL << 53)
#define DESC_UXN        (1ULL << 54)

// MAIR Indices (Assumes init_mair was called)
#define ATTR_DEVICE_IDX 0
#define ATTR_NORMAL_IDX 1

#define L0_SHIFT 39
#define L1_SHIFT 30
#define L2_SHIFT 21
#define L3_SHIFT 12
#define IDX_MASK 0x1FF

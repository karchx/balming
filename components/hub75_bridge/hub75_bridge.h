#pragma once
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int8_t r1, g1, b1, r2, g2, b2;
    int8_t a, b, c, d, e;
    int8_t lat, oe, clk;
    uint16_t width;
    uint16_t height;
    bool double_buff;
} hub75_config_t;

bool begin(const hub75_config_t *cfg);

#ifdef __cplusplus
}
#endif

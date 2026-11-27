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
    uint8_t shift_driver;      // 0 = SHIFTREG (default)
    uint8_t latch_blanking;
    bool clkphase;
    uint8_t color_depth;
    uint32_t i2s_hz;
    uint16_t min_refresh_rate;
} hub75_config_t;

bool begin(const hub75_config_t *cfg);
void draw_pixel(int16_t x, int16_t y, uint16_t color);
void fill_screen(uint16_t color);
void set_brightness8(uint8_t brightness);
void flip_dma_buffer();
int get_refresh_rate(void);

#ifdef __cplusplus
}
#endif

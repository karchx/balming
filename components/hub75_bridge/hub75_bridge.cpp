#include "hub75_bridge.h"
#include "ESP32-HUB75-MatrixPanel-I2S-DMA.h"
#include <cstdint>

static MatrixPanel_I2S_DMA *dma_display = nullptr;

extern "C" {
    bool begin(const hub75_config_t *cfg) {
        if (dma_display != nullptr) return true;

        HUB75_I2S_CFG::i2s_pins pins = {
            cfg->r1, cfg->g1, cfg->b1, cfg->r2, cfg->g2, cfg->b2,
            cfg->a, cfg->b, cfg->c, cfg->d, cfg->e,
            cfg->lat, cfg->oe, cfg->clk
        };

        HUB75_I2S_CFG mxconfig(cfg->width, cfg->height, 1, pins);
        mxconfig.double_buff = cfg->double_buff;
        mxconfig.clkphase = false;

        dma_display = new MatrixPanel_I2S_DMA(mxconfig);

        if (!dma_display->begin()) {
            delete dma_display;
            dma_display = nullptr;
            return false;
        }
        // true CP437 glyph indices — textdraw.h needs them
        // IBM PC / ASCII extend
        // dma_display->cp437(true);
        return true;
    }

    void draw_pixel(int16_t x, int16_t y, uint16_t color) {
        if (dma_display) dma_display->drawPixel(x, y, color);
    }

    void fill_screen(uint16_t color) {
        if (!dma_display) return;
        
        if (color == 0) {
            dma_display->clearScreen();
        } else {
            uint8_t r = ((color >> 11) & 0X1F) << 3;
            uint8_t g = ((color >> 5) & 0X3F) << 2;
            uint8_t b = (color & 0x1F)  << 3;
            dma_display->fillScreenRGB888(r, g, b);
        }
    }

    void set_brightness8(uint8_t brightness) {
        if (dma_display) dma_display->setBrightness8(brightness);
    }

    
    void flip_dma_buffer() {
        if (dma_display) dma_display->flipDMABuffer();
    }

    int get_refresh_rate(void) {
        return dma_display ? dma_display->calculated_refresh_rate : 0;
    }
}


#include "hub75_bridge.h"
#include "ESP32-HUB75-MatrixPanel-I2S-DMA.h"

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
        return true;
    }
}


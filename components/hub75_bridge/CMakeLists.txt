file(GLOB_RECURSE HUB75_SRCS "ESP32-HUB75-MatrixPanel-DMA/src/*.cpp")

idf_component_register(
    SRCS "hub75_bridge.cpp" ${HUB75_SRCS}
    INCLUDE_DIRS "." "ESP32-HUB75-MatrixPanel-DMA/src"
    REQUIRES driver esp_lcd esp_timer
)

target_compile_definitions(${COMPONENT_LIB} PUBLIC NO_GFX=1)


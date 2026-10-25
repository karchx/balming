PROJECT_NAME := balming
TARGET_ARCH   := xtensa-esp32s3-espidf
BUILD_PROFILE := debug

BIN_PATH      := target/$(TARGET_ARCH)/$(BUILD_PROFILE)/$(PROJECT_NAME)
PARTITIONS    := partitions.csv

GET_IDF := . $$HOME/export-esp.sh

.PHONY: all build run refresh_partition erase_flash

all: build run

build:
	@echo "Compile..."
	$(GET_IDF) && cargo build

run:
	$(GET_IDF) && cargo run

refresh_partition: build erase_flash
	@echo "Flash partitions and binary..."
	$(GET_IDF) && espflash flash \
		--partition-table $(PARTITIONS) --monitor \
		$(BIN_PATH)

erase_flash:
	@echo "Clean flash..."
	$(GET_IDF) && espflash erase-flash


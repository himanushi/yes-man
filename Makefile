# Yes Man - M5Stack Core S3 Makefile

# デフォルトシリアルポート (環境に応じて変更)
PORT ?= $(shell ls /dev/cu.usbmodem* 2>/dev/null | head -1)

.PHONY: all build flash monitor run clean check clippy fmt size help

# デフォルトターゲット
all: build

# リリースビルド
build:
	cargo build --release

# デバッグビルド
build-debug:
	cargo build

# デバイスにフラッシュ
flash: build
	@if [ -z "$(PORT)" ]; then \
		echo "Error: No USB device found. Connect M5Stack Core S3."; \
		exit 1; \
	fi
	cargo espflash flash --release --port $(PORT)

# シリアルモニター
monitor:
	@if [ -z "$(PORT)" ]; then \
		echo "Error: No USB device found. Connect M5Stack Core S3."; \
		exit 1; \
	fi
	cargo espflash monitor --port $(PORT)

# フラッシュ + モニター
run: build
	@if [ -z "$(PORT)" ]; then \
		echo "Error: No USB device found. Connect M5Stack Core S3."; \
		exit 1; \
	fi
	cargo espflash flash --release --port $(PORT) --monitor

# ビルド成果物削除
clean:
	cargo clean

# コンパイルチェック (ビルドせずにエラー確認)
check:
	cargo check --release

# Clippy lint チェック
clippy:
	cargo clippy --release -- -W clippy::all

# コードフォーマット
fmt:
	cargo fmt

# フォーマットチェック (CI用)
fmt-check:
	cargo fmt -- --check

# バイナリサイズ確認
size: build
	@echo "=== Binary Size ==="
	@ls -lh target/xtensa-esp32s3-espidf/release/yes-man 2>/dev/null || echo "Binary not found"
	@echo ""
	@cargo espflash board-info --port $(PORT) 2>/dev/null || true

# 利用可能なシリアルポート表示
ports:
	@echo "Available USB ports:"
	@ls /dev/cu.usb* 2>/dev/null || echo "No USB devices found"

# ヘルプ
help:
	@echo "Yes Man - M5Stack Core S3"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build       - Build release binary"
	@echo "  build-debug - Build debug binary"
	@echo "  flash       - Flash to device"
	@echo "  monitor     - Open serial monitor"
	@echo "  run         - Flash and monitor"
	@echo "  clean       - Remove build artifacts"
	@echo "  check       - Check compilation without building"
	@echo "  clippy      - Run clippy lints"
	@echo "  fmt         - Format code"
	@echo "  fmt-check   - Check code formatting"
	@echo "  size        - Show binary size"
	@echo "  ports       - List available USB ports"
	@echo "  help        - Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  PORT        - Serial port (default: auto-detect)"
	@echo ""
	@echo "Examples:"
	@echo "  make run"
	@echo "  make flash PORT=/dev/cu.usbmodem2101"

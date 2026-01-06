# Yes Man - M5Stack Core S3

Fallout: New Vegas の Yes Man を M5Stack Core S3 上に実装するプロジェクト。

## プロジェクト概要

- **ターゲットデバイス**: M5Stack Core S3 (ESP32-S3)
- **言語**: Rust
- **フレームワーク**: esp-rs (esp-idf-hal, esp-idf-svc)

## 開発環境

### 必要なツール

- Rust (stable + ESP32 ターゲット)
- espup (ESP32 Rust ツールチェーンインストーラー)
- cargo-espflash (フラッシュ書き込みツール)
- ldproxy (リンカプロキシ)

### セットアップ

```bash
# ESP32 Rust ツールチェーンのインストール
cargo install espup
espup install

# 追加ツール
cargo install cargo-espflash
cargo install ldproxy
```

### 環境変数

```bash
# espup が生成する export ファイルを読み込む
source ~/export-esp.sh
```

## プロジェクト構成

```
yes-man/
├── CLAUDE.md           # このファイル
├── Cargo.toml          # プロジェクト設定
├── build.rs            # ビルドスクリプト
├── sdkconfig.defaults  # ESP-IDF 設定
├── .cargo/
│   └── config.toml     # Cargo 設定 (ターゲット指定)
└── src/
    ├── main.rs         # エントリポイント
    ├── display.rs      # ディスプレイ制御 (Yes Man の顔)
    ├── audio.rs        # オーディオ再生
    ├── touch.rs        # タッチ入力
    └── animation.rs    # アニメーション制御
```

## M5Stack Core S3 仕様

- **CPU**: ESP32-S3 (Xtensa LX7 デュアルコア, 240MHz)
- **RAM**: 512KB SRAM + 8MB PSRAM
- **Flash**: 16MB
- **ディスプレイ**: 2.0インチ IPS LCD (320x240, ILI9342C)
- **タッチ**: FT6336U 静電容量式タッチ
- **オーディオ**: AW88298 アンプ + ES7210 ADC
- **I2C アドレス**:
  - AXP2101 (電源管理): 0x34
  - FT6336U (タッチ): 0x38
  - AW88298 (アンプ): 0x36
  - ES7210 (ADC): 0x40

## Yes Man 機能

### 表情アニメーション
- アイドル状態: にこやかな表情で待機
- 話している状態: 口パクアニメーション
- タッチ反応: タップで反応

### 音声
- 特徴的なセリフ再生
- 「Sure thing!」「Absolutely!」など

### インタラクション
- タッチで会話トリガー
- ランダムなセリフ選択

## ビルド & 実行

```bash
# ビルド
cargo build --release

# フラッシュ書き込み & モニター
cargo espflash flash --release --monitor
```

## 参考リンク

- [esp-rs Book](https://esp-rs.github.io/book/)
- [esp-idf-hal](https://github.com/esp-rs/esp-idf-hal)
- [M5Stack Core S3 Docs](https://docs.m5stack.com/en/core/CoreS3)

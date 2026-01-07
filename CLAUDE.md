# Yes Man - M5Stack Core S3

Fallout: New Vegas の Yes Man を M5Stack Core S3 上に実装するプロジェクト。

## 重要な指示

- **言語**: 日本語で対応すること
- **コメント**: コード内のコメントは必ず日本語で記述すること
- **エラーメッセージ**: シリアル出力の文字化け防止のため、エラーメッセージは英語で記述すること

## 参照ドキュメント

@documents/m5stack-core-s3-specs.md
@documents/i2c-register-reference.md
@documents/initialization-sequence.md

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

## アーキテクチャ

### レイヤー構造

```
┌─────────────────────────────────────┐
│         Application Layer           │  ← Yes Man ロジック・状態管理
│              (app/)                 │
├─────────────────────────────────────┤
│          Graphics Layer             │  ← 描画・レンダリング
│           (graphics/)               │
├─────────────────────────────────────┤
│      Hardware Abstraction Layer     │  ← ドライバ・トレイト
│           (hardware/)               │
├─────────────────────────────────────┤
│       Configuration Layer           │  ← 定数・設定値
│            (config/)                │
└─────────────────────────────────────┘
```

### デザインパターン

| パターン | 適用箇所 | 説明 |
|---------|---------|------|
| **Facade** | `Board` | 複雑なハードウェア初期化を単一インターフェースに統合 |
| **Strategy** | `State` enum | 状態ベースの動作切り替え (Idle, Talking, Happy, Thinking) |
| **Dependency Injection** | `traits.rs` | トレイトとモックでテスト可能な設計 |

### プロジェクト構成

```
yes-man/
├── CLAUDE.md              # このファイル
├── Cargo.toml             # プロジェクト設定
├── Makefile               # ビルド・フラッシュコマンド
├── build.rs               # ビルドスクリプト
├── sdkconfig.defaults     # ESP-IDF 設定
├── .cargo/
│   └── config.toml        # Cargo 設定 (ターゲット指定)
└── src/
    ├── main.rs            # エントリポイント (薄い)
    ├── lib.rs             # ライブラリルート
    ├── error.rs           # エラー型定義
    ├── config/            # 設定レイヤー
    │   ├── mod.rs
    │   ├── pins.rs        # GPIO ピン定義
    │   ├── i2c.rs         # I2C アドレス・レジスタ
    │   └── display.rs     # ディスプレイ設定
    ├── hardware/          # HAL レイヤー
    │   ├── mod.rs
    │   ├── traits.rs      # トレイト定義・モック
    │   ├── power.rs       # AXP2101 電源管理
    │   ├── io_expander.rs # AW9523 I/O エキスパンダ
    │   └── board.rs       # Board Facade
    ├── graphics/          # 描画レイヤー
    │   ├── mod.rs
    │   ├── colors.rs      # Yes Man テーマカラー
    │   └── renderer.rs    # レンダラー
    └── app/               # アプリケーションレイヤー
        ├── mod.rs
        └── yes_man.rs     # YesManApp・状態管理
```

## M5Stack Core S3 仕様

- **CPU**: ESP32-S3 (Xtensa LX7 デュアルコア, 240MHz)
- **RAM**: 512KB SRAM + 8MB PSRAM
- **Flash**: 16MB
- **ディスプレイ**: 2.0インチ IPS LCD (320x240, ILI9342C)
- **タッチ**: FT6336U 静電容量式タッチ
- **オーディオ**: AW88298 アンプ + ES7210 ADC

### I2C デバイスアドレス

| デバイス | アドレス | 用途 |
|---------|---------|------|
| BMM150 | 0x10 | 磁力計 |
| GC0308 | 0x21 | カメラ (SCCB) |
| LTR-553ALS-WA | 0x23 | 環境光/近接センサー |
| AXP2101 | 0x34 | 電源管理・バックライト制御 |
| AW88298 | 0x36 | オーディオアンプ |
| FT6336U | 0x38 | タッチコントローラー |
| ES7210 | 0x40 | オーディオ ADC (デュアルマイク) |
| BM8563 | 0x51 | RTC |
| AW9523B | 0x58 | I/O エキスパンダ・リセット制御 |
| BMI270 | 0x68 | IMU (6軸) |

### GPIO ピンマップ

| ピン | 用途 |
|-----|------|
| GPIO11 | I2C SCL |
| GPIO12 | I2C SDA |
| GPIO36 | SPI SCLK |
| GPIO37 | SPI MOSI |
| GPIO3 | SPI CS (LCD) |
| GPIO35 | SPI DC (LCD) |

## ビルド & 実行

### Makefile コマンド

```bash
make build      # リリースビルド
make flash      # デバイスにフラッシュ
make monitor    # シリアルモニター
make run        # フラッシュ + モニター
make clean      # ビルド成果物削除
make check      # コンパイルチェック
make clippy     # Lint チェック
make fmt        # コードフォーマット
make size       # バイナリサイズ確認
```

### 手動コマンド

```bash
# ビルド
cargo build --release

# フラッシュ書き込み & モニター
cargo espflash flash --release --monitor

# ポート指定
cargo espflash flash --release --port /dev/cu.usbmodem2101 --monitor
```

## Yes Man 機能 (予定)

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

## トラブルシューティング

### PSRAM エラー
`sdkconfig.defaults` で PSRAM を無効化済み:
```
CONFIG_SPIRAM=n
```

### シリアルポートが見つからない
```bash
ls /dev/cu.usb*
```

### ダウンロードモード
1. リセットボタンを押しながら電源投入
2. または USB 再接続

## 参考リンク

- [esp-rs Book](https://esp-rs.github.io/book/)
- [esp-idf-hal](https://github.com/esp-rs/esp-idf-hal)
- [M5Stack Core S3 Docs](https://docs.m5stack.com/en/core/CoreS3)
- [mipidsi (Display Driver)](https://docs.rs/mipidsi/)
- [embedded-graphics](https://docs.rs/embedded-graphics/)

# M5Stack Core S3 仕様書

## 概要

M5Stack Core S3 は ESP32-S3 ベースの開発ボード。
タッチスクリーン、カメラ、マイク、スピーカーを搭載。

## ハードウェア仕様

### メインプロセッサ
- **チップ**: ESP32-S3 (Xtensa LX7 デュアルコア)
- **クロック**: 240MHz
- **SRAM**: 512KB
- **PSRAM**: 8MB (OPI)
- **Flash**: 16MB
- **WiFi**: 2.4GHz 802.11 b/g/n
- **Bluetooth**: BLE 5.0

### ディスプレイ
- **サイズ**: 2.0 インチ IPS LCD
- **解像度**: 320 x 240 ピクセル
- **コントローラー**: ILI9342C
- **インターフェース**: SPI
- **バックライト**: AXP2101 DLDO1 経由で制御

### タッチパネル
- **タイプ**: 静電容量式
- **コントローラー**: FT6336U
- **I2C アドレス**: 0x38
- **機能**: マルチタッチ、近接検出

### 電源管理
- **PMU**: AXP2101
- **I2C アドレス**: 0x34
- **バッテリー**: 500mAh LiPo (3.7V)
- **充電**: USB-C 経由
- **外部電源**: DinBase 経由で 9-24V DC 入力対応
- **電源制御回路**: 4系統の電源パス制御

### I/O エキスパンダ
- **チップ**: AW9523B
- **I2C アドレス**: 0x58
- **機能**: LCD リセット、タッチリセット、その他 GPIO 拡張

### カメラ
- **センサー**: GC0308
- **解像度**: 0.3MP (VGA: 640 x 480)
- **インターフェース**: DVP (8-bit パラレル)
- **I2C アドレス**: 0x21 (SCCB)
- **チップ ID**: 0x9B
- **備考**: LTR-553ALS-WA と同一リボンケーブル上に配置

### オーディオ
- **スピーカーアンプ**: AW88298 (16-bit I2S)
  - I2C アドレス: 0x36
- **マイク ADC**: ES7210 (デュアルマイク対応)
  - I2C アドレス: 0x40
- **スピーカー**: 1W 内蔵
- **I2S インターフェース**: 標準 I2S バス使用

### センサー
- **IMU**: BMI270 (6軸)
  - I2C アドレス: 0x68
- **磁力計**: BMM150
  - I2C アドレス: 0x10
- **環境光/近接センサー**: LTR-553ALS-WA
  - I2C アドレス: 0x23
  - 環境光測定範囲: 0.01 lux ~ 64,000 lux
  - バックライト自動調整に使用可能
- **近接センサー (タッチ内蔵)**: FT6336U 内蔵 (タッチコントローラー)

### その他
- **RTC**: BM8563
  - I2C アドレス: 0x51
  - スリープ/タイマー起動機能対応
- **SD カード**: microSD スロット (SPI、LCD と共有)
- **USB**: USB-C (OTG & CDC 対応)

### ボタン
- **電源ボタン**: 左上
  - クリック: 電源 ON
  - 6 秒長押し: 電源 OFF
- **リセットボタン**: 右下
  - クリック: リセット
  - 3 秒長押し: ダウンロードモード (緑 LED 点灯)

### LED
- **電源インジケーター LED**: 緑色
  - ダウンロードモード時に点灯

## ピンマップ

### I2C (内部バス)
| ピン | 機能 |
|-----|------|
| GPIO11 | SCL |
| GPIO12 | SDA |

### SPI (ディスプレイ)
| ピン | 機能 |
|-----|------|
| GPIO36 | SCLK |
| GPIO37 | MOSI |
| GPIO35 | DC |
| GPIO3 | CS |

### SPI (SD カード)
| ピン | 機能 |
|-----|------|
| GPIO36 | SCLK |
| GPIO37 | MOSI |
| GPIO39 | MISO |
| GPIO4 | CS |

### I2S (オーディオ)
| ピン | 機能 |
|-----|------|
| GPIO0 | MCLK (マスタークロック) |
| GPIO34 | BCLK |
| GPIO33 | LRCK |
| GPIO13 | DOUT (スピーカー) |
| GPIO14 | DIN (マイク) |

### カメラ (DVP)
| ピン | 機能 |
|-----|------|
| GPIO40 | PCLK |
| GPIO41 | VSYNC |
| GPIO42 | HREF |
| GPIO43 | XCLK |
| GPIO44-51 | D0-D7 |

### ユーザー GPIO (PORT-A)
| ピン | 機能 |
|-----|------|
| GPIO1 | SDA (外部 I2C) |
| GPIO2 | SCL (外部 I2C) |

### ユーザー GPIO (PORT-B)
| ピン | 機能 |
|-----|------|
| GPIO8 | GPIO/ADC |
| GPIO9 | GPIO/ADC |

### ユーザー GPIO (PORT-C)
| ピン | 機能 |
|-----|------|
| GPIO17 | TX |
| GPIO18 | RX |

### タッチ割り込み
| ピン | 機能 |
|-----|------|
| GPIO21 | タッチ IRQ |

## I2C デバイス一覧

| アドレス | デバイス | 機能 |
|---------|---------|------|
| 0x10 | BMM150 | 磁力計 |
| 0x21 | GC0308 | カメラ (SCCB) |
| 0x23 | LTR-553ALS-WA | 環境光/近接センサー |
| 0x34 | AXP2101 | 電源管理 |
| 0x36 | AW88298 | オーディオアンプ |
| 0x38 | FT6336U | タッチ/近接 |
| 0x40 | ES7210 | マイク ADC |
| 0x51 | BM8563 | RTC |
| 0x58 | AW9523B | I/O エキスパンダ |
| 0x68 | BMI270 | IMU (加速度/ジャイロ) |

## AXP2101 電源レール

| レール | 用途 | 電圧 |
|-------|------|------|
| DCDC1 | ESP32-S3 コア | 3.3V |
| DCDC2 | - | - |
| DCDC3 | - | - |
| ALDO1 | - | - |
| ALDO2 | - | - |
| ALDO3 | - | - |
| ALDO4 | - | - |
| BLDO1 | - | - |
| BLDO2 | - | - |
| DLDO1 | LCD バックライト | 0.5-3.3V |
| DLDO2 | - | - |

## AW9523B ピン割り当て

### P0 (0x02 レジスタ)
| ビット | ピン | 機能 |
|-------|------|------|
| 0 | P0_0 | TOUCH_RST |
| 1 | P0_1 | BUS_OUT_EN (電源制御) |
| 2 | P0_2 | USB_OTG_EN (電源制御) |
| 3 | P0_3 | - |
| 4 | P0_4 | - |
| 5 | P0_5 | - |
| 6 | P0_6 | - |
| 7 | P0_7 | - |

### P1 (0x03 レジスタ)
| ビット | ピン | 機能 |
|-------|------|------|
| 0 | P1_0 | CAM_RST |
| 1 | P1_1 | LCD_RST |
| 2 | P1_2 | - |
| 3 | P1_3 | - |
| 4 | P1_4 | - |
| 5 | P1_5 | - |
| 6 | P1_6 | - |
| 7 | P1_7 | - |

## FT6336U タッチコントローラー

### レジスタ
| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | DEV_MODE | デバイスモード |
| 0x01 | GEST_ID | ジェスチャー ID |
| 0x02 | TD_STATUS | タッチポイント数 |
| 0x03-0x08 | P1_* | タッチポイント 1 |
| 0x09-0x0E | P2_* | タッチポイント 2 |
| 0xA0 | ID_G_THGROUP | タッチ閾値 |
| 0xA4 | ID_G_PERIODACTIVE | レポートレート |
| 0xA6 | ID_G_LIB_VERSION_H | ライブラリバージョン |
| 0xA8 | ID_G_CIPHER | チップ ID |
| 0xB0 | ID_G_MODE | 近接検出モード |

### 近接検出
- レジスタ 0xB0 で近接検出モードを有効化
- 0x01: 近接検出有効
- 近接検出時はタッチデータの代わりに近接値を返す

## GC0308 カメラセンサー

### レジスタ
| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | CHIP_ID | チップ ID (0x9B) |

### カメラ初期化
カメラは M5CoreS3 ライブラリ経由で初期化することを推奨。
```cpp
#include "M5CoreS3.h"
#include "esp_camera.h"

CoreS3.Camera.begin();
CoreS3.Camera.sensor->set_framesize(CoreS3.Camera.sensor, FRAMESIZE_QVGA);
```

## 参考リンク

- [M5Stack Core S3 公式ドキュメント](https://docs.m5stack.com/en/core/CoreS3)
- [回路図 (GitHub)](https://github.com/m5stack/M5CoreS3)
- [回路図 PDF](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/datasheet/core/K128%20CoreS3/Sch_M5_CoreS3_v1.0.pdf)
- [ESP32-S3 データシート](https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf)
- [AXP2101 データシート](https://github.com/m5stack/M5-Schematic/blob/master/datasheet/AXP2101.pdf)
- [AW9523B データシート](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/products/core/CoreS3/AW9523B-EN.pdf)
- [ILI9342C データシート](https://www.displayfuture.com/Display/datasheet/controller/ILI9342C.pdf)
- [FT6336U データシート](https://www.buydisplay.com/download/ic/FT6336U.pdf)
- [LTR-553ALS-WA データシート](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/datasheet/core/K128%20CoreS3/LTR-553ALS-WA.PDF)
- [GC0308 データシート](https://m5stack.oss-cn-shenzhen.aliyuncs.com/resource/docs/datasheet/core/K128%20CoreS3/GC0308.pdf)

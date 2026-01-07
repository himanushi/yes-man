# M5Stack Core S3 初期化シーケンス

## 概要

M5Stack Core S3 のハードウェア初期化は特定の順序で行う必要がある。

## 初期化順序

```
1. I2C バス初期化
       ↓
2. AXP2101 (電源管理)
   - バックライト有効化
       ↓
3. AW9523B (I/O エキスパンダ)
   - LCD リセット (P1.1)
   - タッチリセット (P0.0)
   - カメラリセット (P1.0) ※カメラ使用時
       ↓
4. SPI バス初期化
       ↓
5. ILI9342C (ディスプレイ)
       ↓
6. FT6336U (タッチ) ※必要に応じて
       ↓
7. GC0308 (カメラ) ※必要に応じて
       ↓
8. その他デバイス (IMU, RTC, センサー等)
```

## 詳細シーケンス

### Step 1: I2C 初期化

```rust
let i2c_config = I2cConfig::new().baudrate(100.kHz().into());
let i2c = I2cDriver::new(
    peripherals.i2c0,
    peripherals.pins.gpio12,  // SDA
    peripherals.pins.gpio11,  // SCL
    &i2c_config,
)?;
```

**注意:**
- ボーレートは 100kHz を推奨 (安定性のため)
- 400kHz も動作するが、長いケーブルでは問題が起きる可能性

### Step 2: AXP2101 電源初期化

```rust
// チップ ID 確認
let chip_id = read_register(0x34, 0x03)?;
assert_eq!(chip_id, 0x4A);

// DLDO1 (バックライト) 有効化
let dldo_en = read_register(0x34, 0x90)?;
write_register(0x34, 0x90, dldo_en | 0x80)?;

// 電圧設定 (3.3V)
write_register(0x34, 0x99, 0x1C)?;

delay(50ms);  // 電源安定待ち
```

**重要:**
- バックライトを有効化しないと画面が真っ暗
- 電圧は 0x1C (3.3V) で最大輝度

### Step 3: AW9523B I/O エキスパンダ初期化

```rust
// ソフトリセット
write_register(0x58, 0x7F, 0x00)?;
delay(10ms);

// チップ ID 確認
let chip_id = read_register(0x58, 0x10)?;
assert_eq!(chip_id, 0x23);

// プッシュプルモード
write_register(0x58, 0x11, 0x10)?;

// P0, P1 を出力に設定
write_register(0x58, 0x04, 0x00)?;
write_register(0x58, 0x05, 0x00)?;

// LCD リセット (P1.1)
let p1 = read_register(0x58, 0x03)?;
write_register(0x58, 0x03, p1 & !0x02)?;  // LOW
delay(50ms);
write_register(0x58, 0x03, p1 | 0x02)?;   // HIGH
delay(150ms);

// タッチリセット (P0.0)
let p0 = read_register(0x58, 0x02)?;
write_register(0x58, 0x02, p0 & !0x01)?;  // LOW
delay(10ms);
write_register(0x58, 0x02, p0 | 0x01)?;   // HIGH
delay(50ms);

// カメラリセット (P1.0) ※カメラ使用時
let p1 = read_register(0x58, 0x03)?;
write_register(0x58, 0x03, p1 & !0x01)?;  // LOW
delay(10ms);
write_register(0x58, 0x03, p1 | 0x01)?;   // HIGH
delay(50ms);
```

**重要:**
- LCD リセットは必須
- LOW → HIGH のパルスが必要
- 十分なディレイを入れる
- カメラリセットはカメラ使用時のみ必要

### Step 4: SPI 初期化

```rust
let spi_config = SpiConfig::new()
    .baudrate(40.MHz().into());  // 最大 80MHz、安全のため 40MHz

let spi = SpiDeviceDriver::new_single(
    peripherals.spi2,
    peripherals.pins.gpio36,  // SCLK
    peripherals.pins.gpio37,  // MOSI
    None::<Gpio0>,            // MISO (不要)
    Some(peripherals.pins.gpio3),  // CS
    &SpiDriverConfig::default(),
    &spi_config,
)?;

let dc = PinDriver::output(peripherals.pins.gpio35)?;
```

### Step 5: ILI9342C ディスプレイ初期化

```rust
let di = SPIInterface::new(spi, dc);

let mut display = Builder::new(ILI9342CRgb565, di)
    .orientation(Orientation::new().rotate(Rotation::Deg180))
    .color_order(ColorOrder::Bgr)
    .display_size(320, 240)
    .init(&mut delay)?;
```

**注意:**
- 回転は Deg180 が正位置
- カラーオーダーは BGR

### Step 6: GC0308 カメラ初期化 (オプション)

M5CoreS3 ライブラリ使用時:
```cpp
#include "M5CoreS3.h"
#include "esp_camera.h"

auto cfg = M5.config();
CoreS3.begin(cfg);

if (!CoreS3.Camera.begin()) {
    // カメラ初期化失敗
    return;
}

// フレームサイズ設定
CoreS3.Camera.sensor->set_framesize(
    CoreS3.Camera.sensor, FRAMESIZE_QVGA);
```

**注意:**
- カメラは SCCB (I2C 互換) で 0x21 に応答
- AW9523B の P1.0 でリセット制御
- チップ ID: 0x9B (GC0308_PID)

## タイミング図

```
Time →

I2C Init      |████|
              └─┬──┘
AXP2101 Init     |████|
                 └─┬──┘
                   ↓ (50ms wait)
AW9523B Init        |██████████████████|
                    │  reset   delay   │
                    └─┬────────────────┘
                      ↓ (150ms wait)
SPI Init                |████|
                        └─┬──┘
Display Init               |████████|
                           └────────┘
Ready!
```

## トラブルシューティング

### 画面が真っ暗
1. AXP2101 の DLDO1 が有効か確認
2. 電圧が設定されているか確認
3. I2C 通信が成功しているか確認

### 画面が白い / ノイズ
1. LCD リセットが正しく行われているか
2. SPI の配線確認
3. カラーオーダー (BGR) 確認

### タッチが反応しない
1. AW9523B で TOUCH_RST (P0.0) をリセット
2. FT6336U の I2C アドレス (0x38) 確認
3. タッチ閾値の調整
4. タッチ IRQ ピン (GPIO21) の確認

### カメラが動作しない
1. AW9523B で CAM_RST (P1.0) をリセット
2. GC0308 の I2C アドレス (0x21) 確認
3. チップ ID (0x9B) の確認
4. PSRAM が有効か確認 (カメラバッファに必要)

### I2C デバイスが見つからない
```rust
// I2C スキャン
for addr in 0x08..0x78 {
    let mut buf = [0u8; 1];
    if i2c.read(addr, &mut buf, 10).is_ok() {
        log::info!("Found device at 0x{:02X}", addr);
    }
}
```

期待されるデバイス:
- 0x10 (BMM150)
- 0x21 (GC0308) ※カメラ
- 0x23 (LTR-553ALS-WA) ※環境光/近接センサー
- 0x34 (AXP2101)
- 0x36 (AW88298)
- 0x38 (FT6336U)
- 0x40 (ES7210)
- 0x51 (BM8563)
- 0x58 (AW9523B)
- 0x68 (BMI270)

## 電源オフシーケンス

```rust
// バックライト無効化
let dldo_en = read_register(0x34, 0x90)?;
write_register(0x34, 0x90, dldo_en & !0x80)?;

// システムパワーオフ
// 電源ボタン長押し、または AXP2101 のシャットダウンレジスタ
```

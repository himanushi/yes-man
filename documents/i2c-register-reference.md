# I2C レジスタリファレンス

M5Stack Core S3 の主要 I2C デバイスのレジスタ情報。

## AXP2101 (電源管理) - 0x34

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x03 | CHIP_ID | チップ ID (0x4A) |
| 0x90 | DLDO_ENABLE | DLDO1/DLDO2 有効化 |
| 0x99 | DLDO1_VOLTAGE | DLDO1 電圧設定 |

### DLDO_ENABLE (0x90)
```
Bit 7: DLDO1 有効 (バックライト)
Bit 6: DLDO2 有効
Bit 5-0: Reserved
```

### DLDO1_VOLTAGE (0x99)
```
電圧 = 500mV + (値 × 100mV)
0x00 = 500mV
0x1C = 3300mV (最大)
```

### バックライト制御コード例
```rust
// バックライト有効化
let current = read_register(0x90)?;
write_register(0x90, current | 0x80)?;  // DLDO1 有効
write_register(0x99, 0x1C)?;            // 3.3V
```

---

## AW9523B (I/O エキスパンダ) - 0x58

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x02 | OUTPUT0 | P0 出力状態 |
| 0x03 | OUTPUT1 | P1 出力状態 |
| 0x04 | CONFIG0 | P0 方向 (0=出力) |
| 0x05 | CONFIG1 | P1 方向 (0=出力) |
| 0x10 | CHIP_ID | チップ ID (0x23) |
| 0x11 | CTL | 制御レジスタ |
| 0x7F | SOFT_RESET | ソフトリセット |

### CTL (0x11)
```
Bit 4: P0 プッシュプル (1) / オープンドレイン (0)
```

### LCD リセットシーケンス
```rust
// P1.1 = LCD_RST
let p1 = read_register(0x03)?;
write_register(0x03, p1 & !0x02)?;  // LOW
delay(50ms);
write_register(0x03, p1 | 0x02)?;   // HIGH
delay(150ms);
```

---

## FT6336U (タッチ) - 0x38

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | DEV_MODE | デバイスモード |
| 0x01 | GEST_ID | ジェスチャー ID |
| 0x02 | TD_STATUS | タッチポイント数 (0-2) |
| 0x03 | P1_XH | タッチ1 X 上位 + イベント |
| 0x04 | P1_XL | タッチ1 X 下位 |
| 0x05 | P1_YH | タッチ1 Y 上位 + ID |
| 0x06 | P1_YL | タッチ1 Y 下位 |
| 0xA8 | CHIP_ID | チップ ID |
| 0xB0 | ID_G_MODE | 近接モード |

### TD_STATUS (0x02)
```
Bit 3-0: タッチポイント数 (0, 1, 2)
```

### P1_XH (0x03)
```
Bit 7-6: イベントフラグ
  00: Press Down
  01: Lift Up
  10: Contact
  11: Reserved
Bit 3-0: X座標 上位4ビット
```

### タッチ読み取りコード例
```rust
let mut buf = [0u8; 5];
i2c.write_read(0x38, &[0x02], &mut buf)?;

let touch_count = buf[0] & 0x0F;
if touch_count > 0 {
    let x = ((buf[1] & 0x0F) as u16) << 8 | buf[2] as u16;
    let y = ((buf[3] & 0x0F) as u16) << 8 | buf[4] as u16;
}
```

### 近接検出モード (0xB0)
```
0x00: 無効
0x01: 有効
```

---

## LTR-553ALS-WA (環境光/近接センサー) - 0x23

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x80 | ALS_CONTR | ALS 制御 (ゲイン、動作モード) |
| 0x81 | PS_CONTR | 近接センサー制御 |
| 0x85 | ALS_MEAS_RATE | ALS 測定レート |
| 0x88 | ALS_DATA_CH1_0 | ALS CH1 データ下位 |
| 0x89 | ALS_DATA_CH1_1 | ALS CH1 データ上位 |
| 0x8A | ALS_DATA_CH0_0 | ALS CH0 データ下位 |
| 0x8B | ALS_DATA_CH0_1 | ALS CH0 データ上位 |
| 0x8D | PS_DATA_0 | 近接データ下位 |
| 0x8E | PS_DATA_1 | 近接データ上位 (11-bit) |
| 0x8F | ALS_PS_STATUS | ALS/PS ステータス |
| 0x87 | PART_ID | パート ID |
| 0x86 | MANUFAC_ID | 製造者 ID (0x05) |

### ALS_CONTR (0x80)
```
Bit 7-5: Reserved
Bit 4-2: ALS Gain
  000: 1x (1 ~ 64k lux)
  001: 2x
  010: 4x
  011: 8x
  100: 48x
  110: 96x (0.01 ~ 600 lux)
Bit 1: SW Reset (書き込み後自動クリア)
Bit 0: ALS Mode (0=Standby, 1=Active)
```

### PS_CONTR (0x81)
```
Bit 7-6: Reserved
Bit 5-4: PS Saturation Indicator Enable
Bit 3-2: PS Mode (00=Standby, 11=Active)
Bit 1-0: Reserved
```

### 環境光読み取りコード例
```rust
// ALS 有効化 (1x gain)
write_register(0x23, 0x80, 0x01)?;
delay(100ms);

// データ読み取り (CH1 → CH0 の順で読む)
let mut buf = [0u8; 4];
i2c.write_read(0x23, &[0x88], &mut buf)?;

let ch1 = (buf[1] as u16) << 8 | buf[0] as u16;
let ch0 = (buf[3] as u16) << 8 | buf[2] as u16;

// lux 計算 (簡易版)
let ratio = ch1 as f32 / ch0 as f32;
let lux = if ratio < 0.45 {
    (1.7743 * ch0 as f32 + 1.1059 * ch1 as f32)
} else {
    // ... ゲインとレシオに応じた計算
};
```

### 近接検出読み取りコード例
```rust
// PS 有効化
write_register(0x23, 0x81, 0x03)?;
delay(10ms);

// データ読み取り
let mut buf = [0u8; 2];
i2c.write_read(0x23, &[0x8D], &mut buf)?;

let ps_data = ((buf[1] & 0x07) as u16) << 8 | buf[0] as u16;  // 11-bit
```

---

## BMI270 (IMU) - 0x68

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | CHIP_ID | チップ ID (0x24) |
| 0x0C-0x11 | DATA | 加速度/ジャイロデータ |
| 0x40 | ACC_CONF | 加速度設定 |
| 0x42 | GYR_CONF | ジャイロ設定 |
| 0x7C | PWR_CTRL | 電源制御 |
| 0x7D | PWR_CONF | 電源設定 |

### DATA レジスタ
```
0x0C-0x0D: ACC_X (16-bit, LSB first)
0x0E-0x0F: ACC_Y
0x10-0x11: ACC_Z
0x12-0x13: GYR_X
0x14-0x15: GYR_Y
0x16-0x17: GYR_Z
```

---

## BM8563 (RTC) - 0x51

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | CONTROL1 | 制御1 |
| 0x01 | CONTROL2 | 制御2 |
| 0x02 | SECONDS | 秒 (BCD) |
| 0x03 | MINUTES | 分 (BCD) |
| 0x04 | HOURS | 時 (BCD) |
| 0x05 | DAYS | 日 (BCD) |
| 0x06 | WEEKDAYS | 曜日 |
| 0x07 | MONTHS | 月 (BCD) |
| 0x08 | YEARS | 年 (BCD, 00-99) |

### 時刻読み取りコード例
```rust
let mut buf = [0u8; 7];
i2c.write_read(0x51, &[0x02], &mut buf)?;

let seconds = bcd_to_dec(buf[0] & 0x7F);
let minutes = bcd_to_dec(buf[1] & 0x7F);
let hours = bcd_to_dec(buf[2] & 0x3F);
let days = bcd_to_dec(buf[3] & 0x3F);
let months = bcd_to_dec(buf[5] & 0x1F);
let years = bcd_to_dec(buf[6]) + 2000;

fn bcd_to_dec(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}
```

---

## ES7210 (マイク ADC) - 0x40

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | RESET | ソフトリセット |
| 0x01 | CLK_ON | クロック有効化 |
| 0x02 | MODE_CFG | モード設定 |

---

## AW88298 (スピーカーアンプ) - 0x36

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | CHIP_ID | チップ ID |
| 0x01 | SYSCTRL | システム制御 |
| 0x02 | MODE | モード設定 |
| 0x05 | VOLUME | ボリューム |

### ボリューム設定
```
0x00 = ミュート
0xFF = 最大音量
```

---

## GC0308 (カメラ) - 0x21

GC0308 は SCCB (I2C 互換) インターフェースで制御。
M5CoreS3 ライブラリ経由での使用を推奨。

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x00 | CHIP_ID | チップ ID (0x9B) |

### 備考
- カメラは AW9523B の P1.0 (CAM_RST) でリセット制御
- LTR-553ALS-WA と同一リボンケーブル上に配置
- DVP インターフェース (8-bit パラレル) でデータ転送
- 解像度: VGA (640x480), QVGA (320x240) 等

### 初期化コード例 (M5CoreS3 ライブラリ使用)
```cpp
#include "M5CoreS3.h"
#include "esp_camera.h"

void setup() {
    auto cfg = M5.config();
    CoreS3.begin(cfg);

    if (!CoreS3.Camera.begin()) {
        Serial.println("Camera Init Fail");
        return;
    }

    // フレームサイズ設定
    CoreS3.Camera.sensor->set_framesize(
        CoreS3.Camera.sensor, FRAMESIZE_QVGA);
}

void loop() {
    if (CoreS3.Camera.get()) {
        // フレームバッファ取得成功
        uint8_t* buf = CoreS3.Camera.fb->buf;
        size_t len = CoreS3.Camera.fb->len;

        // 画像処理...

        CoreS3.Camera.free();
    }
}
```

---

## BMM150 (磁力計) - 0x10

BMM150 は BMI270 の Sensor Hub 経由で接続されている場合がある。

### 主要レジスタ

| アドレス | 名前 | 説明 |
|---------|------|------|
| 0x40 | CHIP_ID | チップ ID (0x32) |
| 0x42 | DATA_X_LSB | X軸データ下位 |
| 0x43 | DATA_X_MSB | X軸データ上位 |
| 0x44 | DATA_Y_LSB | Y軸データ下位 |
| 0x45 | DATA_Y_MSB | Y軸データ上位 |
| 0x46 | DATA_Z_LSB | Z軸データ下位 |
| 0x47 | DATA_Z_MSB | Z軸データ上位 |
| 0x4C | CTRL | 電源制御 |

### 電源制御 (0x4C)
```
Bit 1: Power Control (0=Suspend, 1=Active)
```

### 備考
- BMI270 の Sensor Hub 経由で 9 軸統合データ取得可能
- 磁石を含む製品が近くにあると測定に影響

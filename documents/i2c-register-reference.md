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

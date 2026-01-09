//! LTR-553ALS-WA 環境光センサー ドライバ

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cDriver;
use crate::config::i2c::ltr553;
use crate::error::YesManError;

/// LTR-553ALS-WA 環境光センサー操作
pub struct Ltr553;

impl Ltr553 {
    /// 環境光センサーを初期化
    pub fn init(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("LTR-553 を 0x{:02X} で初期化中...", ltr553::ADDR);

        // まずスタンバイモードに設定してセンサーを起動
        log::info!("  スタンバイモードに設定中...");
        Self::write_register(i2c, ltr553::reg::ALS_CONTR, ltr553::als_ctrl::STANDBY)?;
        FreeRtos::delay_ms(10);

        // 製造者 ID を確認
        log::info!("  製造者 ID を読み取り中...");
        let manufac_id = Self::read_register(i2c, ltr553::reg::MANUFAC_ID)?;
        log::info!("  製造者 ID: 0x{:02X} (期待値: 0x{:02X})", manufac_id, ltr553::EXPECTED_MANUFAC_ID);

        // パーツ ID も追加確認
        let part_id = Self::read_register(i2c, ltr553::reg::PART_ID)?;
        log::info!("  パーツ ID: 0x{:02X}", part_id);

        // ALS をアクティブモードで 1x ゲインで有効化 (広範囲: 1 ~ 64k lux)
        log::info!("  ALS アクティブモードを有効化中...");
        let als_ctrl_value = ltr553::als_ctrl::GAIN_1X | ltr553::als_ctrl::ACTIVE;
        Self::write_register(i2c, ltr553::reg::ALS_CONTR, als_ctrl_value)?;

        // 測定レート設定: 100ms 積分時間、500ms 繰り返しレート
        Self::write_register(i2c, ltr553::reg::ALS_MEAS_RATE, 0x03)?;

        // 最初の測定を待つ
        FreeRtos::delay_ms(100);

        // 設定が正しく書き込まれたか確認
        let als_contr_readback = Self::read_register(i2c, ltr553::reg::ALS_CONTR)?;
        log::info!("  ALS_CONTR: wrote 0x{:02X}, read back 0x{:02X}", als_ctrl_value, als_contr_readback);

        // ステータスレジスタを確認
        let status = Self::read_register(i2c, ltr553::reg::ALS_PS_STATUS)?;
        log::info!("  ALS_PS_STATUS: 0x{:02X} (bit2=ALS_DATA_STATUS)", status);

        log::info!("LTR-553 初期化成功");
        Ok(())
    }

    /// 環境光レベルを読み取り (生の ADC 値)
    /// CH0 (可視光 + IR) と CH1 (IR のみ) の値を返す
    pub fn read_als_raw(i2c: &mut I2cDriver) -> Result<(u16, u16), YesManError> {
        // 4 バイトを一度に読み取り (CH1_0, CH1_1, CH0_0, CH0_1)
        let mut buf = [0u8; 4];
        i2c.write_read(ltr553::ADDR, &[ltr553::reg::ALS_DATA_CH1_0], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 read error: {:?}", e)))?;

        let ch1 = u16::from_le_bytes([buf[0], buf[1]]);
        let ch0 = u16::from_le_bytes([buf[2], buf[3]]);

        Ok((ch0, ch1))
    }

    /// 環境光を読み取り、おおよその lux 値を計算
    /// バックライト制御に適した簡易計算を使用
    pub fn read_lux(i2c: &mut I2cDriver) -> Result<u32, YesManError> {
        let (ch0, ch1) = Self::read_als_raw(i2c)?;

        // デバッグ: 生データをログ出力
        log::info!("LTR-553 raw: ch0={}, ch1={}", ch0, ch1);

        // 簡易 lux 計算
        // LTR-553 のデータシートに基づく計算式
        // ratio = CH1 / CH0 で光源の種類を判定
        let lux = if ch0 == 0 {
            0
        } else {
            let ratio = (ch1 as f32) / (ch0 as f32);
            let lux_f = if ratio < 0.45 {
                1.7743 * (ch0 as f32) + 1.1059 * (ch1 as f32)
            } else if ratio < 0.64 {
                4.2785 * (ch0 as f32) - 1.9548 * (ch1 as f32)
            } else if ratio < 0.85 {
                0.5926 * (ch0 as f32) + 0.1185 * (ch1 as f32)
            } else {
                // ratio >= 0.85: 主に IR 光源（白熱灯など）
                // 簡易的に ch0 ベースで計算
                0.1 * (ch0 as f32)
            };
            // 最低でも ch0 が 0 でなければ 1 lux を返す
            (lux_f as u32).max(1)
        };

        Ok(lux)
    }

    /// lux をバックライト明るさ (0-100%) に変換
    /// 自然な知覚のために対数マッピングを使用
    pub fn lux_to_brightness(lux: u32) -> u8 {
        // 人間の目は光を対数的に知覚する
        // lux 範囲を明るさレベルにマッピング:
        // 0-10 lux: 10-30% (暗い部屋)
        // 10-100 lux: 30-50% (薄暗い室内)
        // 100-1000 lux: 50-70% (室内)
        // 1000-10000 lux: 70-90% (明るい室内 / 日陰)
        // 10000+ lux: 90-100% (直射日光)

        const MIN_BRIGHTNESS: u8 = 15;
        const MAX_BRIGHTNESS: u8 = 100;

        if lux == 0 {
            return MIN_BRIGHTNESS;
        }

        // 対数マッピング
        // log10(1) = 0, log10(10) = 1, log10(100) = 2, log10(10000) = 4
        let log_lux = (lux as f32).log10();

        // log10 範囲 [0, 4] を明るさ [10, 100] にマッピング
        let normalized = (log_lux / 4.0).clamp(0.0, 1.0);
        let brightness = MIN_BRIGHTNESS as f32 + normalized * (MAX_BRIGHTNESS - MIN_BRIGHTNESS) as f32;

        brightness as u8
    }

    fn read_register(i2c: &mut I2cDriver, reg: u8) -> Result<u8, YesManError> {
        let mut buf = [0u8; 1];
        i2c.write_read(ltr553::ADDR, &[reg], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 read error: {:?}", e)))?;
        Ok(buf[0])
    }

    fn write_register(i2c: &mut I2cDriver, reg: u8, value: u8) -> Result<(), YesManError> {
        i2c.write(ltr553::ADDR, &[reg, value], 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 write error: {:?}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lux_to_brightness_dark() {
        // 暗い環境のテスト
        assert_eq!(Ltr553::lux_to_brightness(0), 10);
        assert!(Ltr553::lux_to_brightness(1) >= 10);
        assert!(Ltr553::lux_to_brightness(10) <= 50);
    }

    #[test]
    fn test_lux_to_brightness_bright() {
        // 明るい環境のテスト
        assert!(Ltr553::lux_to_brightness(10000) >= 90);
        assert_eq!(Ltr553::lux_to_brightness(100000), 100);
    }

    #[test]
    fn test_lux_to_brightness_monotonic() {
        // 単調増加のテスト
        let b1 = Ltr553::lux_to_brightness(10);
        let b2 = Ltr553::lux_to_brightness(100);
        let b3 = Ltr553::lux_to_brightness(1000);
        assert!(b1 <= b2);
        assert!(b2 <= b3);
    }
}

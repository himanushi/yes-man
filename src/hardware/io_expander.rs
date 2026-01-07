//! AW9523B I/O エキスパンダ ドライバ

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cDriver;
use crate::config::i2c::aw9523;
use crate::error::YesManError;

/// AW9523B I/O エキスパンダ操作
pub struct Aw9523;

impl Aw9523 {
    /// AW9523 を初期化
    pub fn init(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("AW9523 を 0x{:02X} で初期化中...", aw9523::ADDR);

        // ソフトリセット
        let _ = Self::write_register(i2c, aw9523::reg::SOFT_RESET, 0x00);
        FreeRtos::delay_ms(10);

        // チップ ID 確認
        let chip_id = Self::read_register(i2c, aw9523::reg::CHIP_ID)?;
        log::info!("AW9523 チップ ID: 0x{:02X} (期待値: 0x{:02X})", chip_id, aw9523::EXPECTED_CHIP_ID);

        // 制御レジスタ設定 - プッシュプルモード
        Self::write_register(i2c, aw9523::reg::CTL, aw9523::ctl::PUSH_PULL)?;

        // P0 と P1 を出力に設定 (0 = 出力)
        Self::write_register(i2c, aw9523::reg::CONFIG0, 0x00)?;
        Self::write_register(i2c, aw9523::reg::CONFIG1, 0x00)?;

        // 初期出力状態を設定:
        // P0: TOUCH_RST(0)=HIGH, BUS_OUT_EN(1)=HIGH
        // P1: CAM_RST(0)=HIGH, LCD_RST(1)=HIGH
        Self::write_register(i2c, aw9523::reg::OUTPUT0, 0x03)?; // P0.0, P0.1 を HIGH
        Self::write_register(i2c, aw9523::reg::OUTPUT1, 0x03)?; // P1.0, P1.1 を HIGH

        log::info!("AW9523 初期化完了 (BUS_OUT_EN 有効)");
        Ok(())
    }

    /// 周辺機器バス出力を有効化
    pub fn enable_bus_output(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("周辺機器バス出力 (P0.1) を有効化中...");
        const BUS_OUT_EN: u8 = 1 << 1; // P0.1

        let mut p0_state = Self::read_register(i2c, aw9523::reg::OUTPUT0).unwrap_or(0);
        p0_state |= BUS_OUT_EN;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;

        log::info!("周辺機器バス有効化完了");
        Ok(())
    }

    /// I/O エキスパンダ経由で LCD をリセット
    pub fn reset_lcd(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("LCD をリセット中...");

        // 現在の P1 状態を読み取り
        let mut p1_state = Self::read_register(i2c, aw9523::reg::OUTPUT1).unwrap_or(0);

        // LCD_RST を LOW に
        p1_state &= !aw9523::pins::LCD_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(50);

        // LCD_RST を HIGH に
        p1_state |= aw9523::pins::LCD_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(150);

        log::info!("LCD リセット完了");
        Ok(())
    }

    /// I/O エキスパンダ経由でタッチコントローラをリセット (P0.0)
    pub fn reset_touch(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("タッチコントローラをリセット中...");

        // TOUCH_RST は P0.0
        const TOUCH_RST: u8 = 1 << 0;

        // 現在の P0 状態を読み取り
        let mut p0_state = Self::read_register(i2c, aw9523::reg::OUTPUT0).unwrap_or(0);

        // TOUCH_RST を LOW に
        p0_state &= !TOUCH_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;
        FreeRtos::delay_ms(10);

        // TOUCH_RST を HIGH に
        p0_state |= TOUCH_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;
        FreeRtos::delay_ms(50);

        log::info!("タッチリセット完了");
        Ok(())
    }

    /// I/O エキスパンダ経由でカメラをリセット (P1.0)
    /// LTR-553 と同じリボンケーブルを共有しているため、センサーアクセスにも必要
    pub fn reset_camera(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("カメラをリセット中 (LTR-553 アクセス用)...");

        // CAM_RST は P1.0
        const CAM_RST: u8 = 1 << 0;

        // 現在の P1 状態を読み取り
        let mut p1_state = Self::read_register(i2c, aw9523::reg::OUTPUT1).unwrap_or(0);

        // CAM_RST を LOW に
        p1_state &= !CAM_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(10);

        // CAM_RST を HIGH に
        p1_state |= CAM_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(50);

        log::info!("カメラリセット完了");
        Ok(())
    }

    /// ピンを HIGH に設定
    pub fn set_pin_high(i2c: &mut I2cDriver, pin: u8) -> Result<(), YesManError> {
        if pin < 8 {
            // P0 ピン
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT0)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT0, current | (1 << pin))?;
        } else {
            // P1 ピン
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT1)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT1, current | (1 << (pin - 8)))?;
        }
        Ok(())
    }

    /// ピンを LOW に設定
    pub fn set_pin_low(i2c: &mut I2cDriver, pin: u8) -> Result<(), YesManError> {
        if pin < 8 {
            // P0 ピン
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT0)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT0, current & !(1 << pin))?;
        } else {
            // P1 ピン
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT1)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT1, current & !(1 << (pin - 8)))?;
        }
        Ok(())
    }

    /// レジスタ読み取り
    fn read_register(i2c: &mut I2cDriver, reg: u8) -> Result<u8, YesManError> {
        let mut buf = [0u8; 1];
        i2c.write_read(aw9523::ADDR, &[reg], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("AW9523 read error: {:?}", e)))?;
        Ok(buf[0])
    }

    /// レジスタ書き込み
    fn write_register(i2c: &mut I2cDriver, reg: u8, value: u8) -> Result<(), YesManError> {
        i2c.write(aw9523::ADDR, &[reg, value], 100)
            .map_err(|e| YesManError::I2c(format!("AW9523 write error: {:?}", e)))?;
        Ok(())
    }
}

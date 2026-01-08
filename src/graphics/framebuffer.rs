//! フレームバッファによるダブルバッファリング描画
//!
//! 描画をメモリ上で行い、flush() で一括転送することで
//! ちらつきのない滑らかな画面更新を実現する。

use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
};

use crate::config::display::{WIDTH, HEIGHT};

/// フレームバッファのピクセル数
const PIXEL_COUNT: usize = WIDTH as usize * HEIGHT as usize;

/// フレームバッファ
///
/// 320x240 の Rgb565 ピクセルをメモリ上に保持し、
/// embedded-graphics の描画ターゲットとして機能する。
///
/// # 使用例
///
/// ```ignore
/// let mut fb = FrameBuffer::new();
/// fb.clear(Rgb565::BLACK);
/// Circle::new(Point::new(100, 100), 50)
///     .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
///     .draw(&mut fb)?;
/// fb.flush(&mut display)?;
/// ```
pub struct FrameBuffer {
    /// ピクセルバッファ (320 * 240 * 2 = 153,600 bytes)
    buffer: Box<[u16; PIXEL_COUNT]>,
}

impl FrameBuffer {
    /// 新しいフレームバッファを作成（黒で初期化）
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0u16; PIXEL_COUNT]),
        }
    }

    /// 指定色でフレームバッファ全体をクリア
    pub fn clear_with(&mut self, color: Rgb565) {
        let raw = color.into_storage();
        self.buffer.fill(raw);
    }

    /// フレームバッファの内容をディスプレイに一括転送
    ///
    /// DrawTarget を実装した任意のディスプレイに転送可能。
    /// mipidsi の Display を渡すことを想定。
    pub fn flush<D>(&self, display: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb565>,
    {
        // フレームバッファの全ピクセルをイテレータとして生成
        let pixels = self.buffer.iter().enumerate().map(|(i, &raw)| {
            let x = (i % WIDTH as usize) as i32;
            let y = (i / WIDTH as usize) as i32;
            Pixel(Point::new(x, y), Rgb565::from(RawU16::new(raw)))
        });

        display.draw_iter(pixels)
    }

    /// 生のバッファへの参照を取得（高速転送用）
    pub fn as_raw(&self) -> &[u16; PIXEL_COUNT] {
        &self.buffer
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// DrawTarget トレイトの実装
///
/// embedded-graphics の描画プリミティブ（Circle, Rectangle, Text など）を
/// フレームバッファに描画可能にする。
impl DrawTarget for FrameBuffer {
    type Color = Rgb565;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels.into_iter() {
            // 境界チェック
            if point.x >= 0
                && point.x < WIDTH as i32
                && point.y >= 0
                && point.y < HEIGHT as i32
            {
                let index = (point.y as usize) * (WIDTH as usize) + (point.x as usize);
                self.buffer[index] = color.into_storage();
            }
        }
        Ok(())
    }
}

/// OriginDimensions トレイトの実装
///
/// フレームバッファのサイズを embedded-graphics に通知する。
impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics::primitives::{Circle, PrimitiveStyle};

    #[test]
    fn test_new_framebuffer_is_black() {
        let fb = FrameBuffer::new();
        assert!(fb.buffer.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_clear_with_color() {
        let mut fb = FrameBuffer::new();
        fb.clear_with(Rgb565::WHITE);
        let white = Rgb565::WHITE.into_storage();
        assert!(fb.buffer.iter().all(|&p| p == white));
    }

    #[test]
    fn test_draw_pixel() {
        let mut fb = FrameBuffer::new();
        Pixel(Point::new(10, 20), Rgb565::RED)
            .draw(&mut fb)
            .unwrap();

        let index = 20 * WIDTH as usize + 10;
        assert_eq!(fb.buffer[index], Rgb565::RED.into_storage());
    }

    #[test]
    fn test_draw_circle() {
        let mut fb = FrameBuffer::new();
        Circle::new(Point::new(100, 100), 10)
            .into_styled(PrimitiveStyle::with_fill(Rgb565::BLUE))
            .draw(&mut fb)
            .unwrap();

        // 中心付近のピクセルが青になっているはず
        let center_index = 105 * WIDTH as usize + 105;
        assert_eq!(fb.buffer[center_index], Rgb565::BLUE.into_storage());
    }

    #[test]
    fn test_out_of_bounds_ignored() {
        let mut fb = FrameBuffer::new();
        // 境界外のピクセルは無視される
        Pixel(Point::new(-1, 0), Rgb565::RED).draw(&mut fb).unwrap();
        Pixel(Point::new(0, -1), Rgb565::RED).draw(&mut fb).unwrap();
        Pixel(Point::new(320, 0), Rgb565::RED).draw(&mut fb).unwrap();
        Pixel(Point::new(0, 240), Rgb565::RED).draw(&mut fb).unwrap();

        // バッファは変更されていない
        assert!(fb.buffer.iter().all(|&p| p == 0));
    }
}

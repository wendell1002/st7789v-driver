use embedded_graphics_core::pixelcolor::Rgb565;
use embedded_graphics_core::prelude::{DrawTarget, OriginDimensions, Size};
use embedded_graphics_core::Pixel;
use embedded_hal::blocking::spi::*;
use embedded_hal::digital::v2::OutputPin;

use crate::st7789::ST7789;

// Implementing the DrawTarget trait for the ST7789V2 display driver
impl<SPI, DC, CS, RST, BLK> DrawTarget for ST7789<SPI, DC, CS, RST, BLK>
where
    SPI: Write<u8> + Transfer<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        use crate::batch::DrawBatch;
        self.draw_batch(pixels)
        // for Pixel(coord, color) in pixels.into_iter() {
        //     let color_value = color.into_storage();
        //     // info!("draw pixel: {} {}", coord.x, coord.y);
        //     // Only draw pixels that would be on screen
        //     if coord.x >= 0
        //         && coord.y >= 0
        //         && coord.x < self.width as i32
        //         && coord.y < self.height as i32
        //     {
        //         self.write_pixel(coord.x as u16, coord.y as u16, color_value)?;
        //     }
        // }
    }
}

// Implementing the OriginDimensions trait for the ST7789V2 display driver
impl<SPI, DC, CS, RST, BLK> OriginDimensions for ST7789<SPI, DC, CS, RST, BLK>
where
    SPI: Write<u8> + Transfer<u8>,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

use crate::st7789::st7789::ST7789;
use embedded_hal::blocking::spi::*;
use embedded_hal::digital::v2::OutputPin;

pub trait Block {}

impl< DC, CS, RST, BLK> Block for ST7789<  DC, CS, RST, BLK>
where
 
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
}

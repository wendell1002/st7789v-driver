use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiDevice;

use crate::cmd::Commands;
use crate::region::Region;

pub const HORIZONTAL: u16 = 0;
pub const VERTICAL: u16 = 1;

///
/// Display orientation.
///
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Orientation {
    Portrait = 0b0000_0000,         // no inverting
    Landscape = 0b0110_0000,        // invert column and page/column order
    PortraitSwapped = 0b1100_0000,  // invert page and column order
    LandscapeSwapped = 0b1010_0000, // invert page and page/column order
}

impl Default for Orientation {
    fn default() -> Self {
        Self::Portrait
    }
}

/// Driver for the ST7789 display.
///
/// ```rust
/// #![no_std]
///     #![no_main]
///     
///     use core::convert::Infallible;
///     
///     use defmt::*;
///     use embassy_executor::Spawner;
///     use embassy_spi::st7789::st7789::{Orientation, ST7789};
///     use embassy_stm32::{
///         gpio::{Level, Output, Speed},
///         rcc::clocks,
///         spi::{self, Config, MisoPin, Spi},
///         time::Hertz,
///     };
///     use embassy_time::{Delay, Instant, Timer};
///     use embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
///     use embedded_graphics::prelude::RgbColor as _;
///     use embedded_graphics::prelude::RgbColor;
///     use embedded_graphics::primitives::*;
///     use embedded_graphics::primitives::{PrimitiveStyleBuilder, RoundedRectangle};
///     use embedded_graphics::Drawable;
///     use embedded_graphics::{
///         mono_font::ascii::{FONT_6X10, FONT_6X9},
///         prelude::DrawTargetExt,
///     };
///     use embedded_graphics::{mono_font::MonoTextStyleBuilder, primitives::PrimitiveStyle};
///     use embedded_graphics::{
///         mono_font::{ascii::FONT_9X15, iso_8859_15::FONT_10X20},
///         prelude::{IntoStorage, Point, Primitive, Size},
///         primitives::{Line, Polyline, Rectangle, Triangle},
///         text::{Baseline, Text},
///         Pixel,
///     };
///     use embedded_graphics_core::draw_target::DrawTarget;
///     use embedded_hal::digital::{ErrorType, OutputPin};
///     use embedded_hal_bus::spi::ExclusiveDevice;
///     use heapless::String;
///     use {defmt_rtt as _, panic_probe as _};
///     #[embassy_executor::main]
///     async fn main(_spawner: Spawner) {
///         let mut config = embassy_stm32::Config::default();
///         {
///             use embassy_stm32::rcc::*;
///             config.rcc.hse = Some(Hse {
///                 freq: Hertz(8_000_000),
///                 mode: HseMode::Oscillator,
///             });
///             config.rcc.pll = Some(Pll {
///                 prediv: PllPreDiv::DIV1,
///                 mul: PllMul::MUL9,
///                 src: PllSource::HSE,
///             });
///             config.rcc.ahb_pre = AHBPrescaler::DIV1;
///             config.rcc.apb1_pre = APBPrescaler::DIV1;
///             config.rcc.apb2_pre = APBPrescaler::DIV1;
///             config.rcc.sys = Sysclk::PLL1_P;
///         }
///         let p = embassy_stm32::init(config);
///         info!("RCC: {:?}", clocks(&p.RCC));
///     
///         let mut spi_config = Config::default();
///         spi_config.frequency = Hertz(36_000_000);
///         spi_config.mode = spi::Mode {
///             polarity: spi::Polarity::IdleLow,
///             phase: spi::Phase::CaptureOnFirstTransition,
///         };
///         spi_config.gpio_speed = Speed::VeryHigh;
///     
///         let spi = Spi::new(
///             p.SPI2, p.PB13, p.PB15, p.PB14, p.DMA1_CH5, p.DMA1_CH4, spi_config,
///         );
///     
///         let cs = Output::new(p.PA10, Level::High, Speed::VeryHigh);
///         let dc = Output::new(p.PA9, Level::High, Speed::VeryHigh);
///         let rst = Output::new(p.PA8, Level::High, Speed::VeryHigh);
///         let blk = Output::new(p.PB12, Level::High, Speed::VeryHigh);
///         let delay = Delay;
///         let spi_device = ExclusiveDevice::new(spi, cs, delay).unwrap();
///     
///         let mut display = ST7789::new(
///             spi_device,
///             dc,
///             NoCsPin,
///             Some(rst),
///             Some(blk),
///             true,
///             240,
///             135,
///         );
///         display
///             .set_orientation(Orientation::Landscape)
///             .await
///             .unwrap();
///         display.set_offset(52, 40);
///         // initialize
///         display.init(&mut Delay {}).await.unwrap();
///     
///         info!("init displayer3");
///         // display.clear_regions();
///         // display.clear_screen(Rgb565::YELLOW.).unwrap();
///         // set default orientation
///         display.clear(Rgb565::BLACK).unwrap();
///     
///         info!("draw");
///         let style = PrimitiveStyleBuilder::new()
///             .stroke_width(5)
///             .stroke_color(Rgb565::GREEN)
///             .fill_color(Rgb565::BLACK)
///             .build();
///     
///         RoundedRectangle::with_equal_corners(
///             Rectangle::new(Point::new(14, 16), Size::new(100, 40)),
///             Size::new(12, 12),
///         )
///         .into_styled(style)
///         .draw(&mut display)
///         .unwrap();
///         let text_style = MonoTextStyleBuilder::new()
///             .font(&FONT_10X20)
///             .text_color(Rgb565::YELLOW)
///             .build();
///         Text::with_baseline("Hello", Point::new(43, 26), text_style, Baseline::Top)
///             .draw(&mut display)
///             .unwrap();
///         let mut i = 0;
///         use core::fmt::Write;
///         let line_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);
///         let text_style = MonoTextStyleBuilder::new()
///             .font(&FONT_10X20)
///             .text_color(Rgb565::WHITE)
///             .build();
///         // let color = Rgb565::RED;
///         // display.set_pixel(12, 12, color).unwrap();
///     
///         let mut fps = 0;
///     
///         let colors = [
///             Rgb565::RED,
///             Rgb565::GREEN,
///             Rgb565::BLUE,
///             Rgb565::YELLOW,
///             Rgb565::MAGENTA,
///             Rgb565::CYAN,
///         ];
///         let size = 1000;
///         let mut sp_str = String::<24>::new();
///         loop {
///             let now = Instant::now();
///             for j in 0..size {
///                 display
///                     .clear_screen(colors[j % colors.len()].into_storage())
///                     .await
///                     .unwrap();
///                 // display.clear(colors[j % colors.len()]).unwrap();
///             }
///             fps = 1000 * size as usize / now.elapsed().as_millis() as usize;
///             info!("FPS:{}", fps);
///             sp_str.clear();
///     
///             // for y in 0..135 {
///             //     display
///             //         .set_pixels(0, y, 240, 135, [Rgb565::BLUE; 240].into_iter())
///             //         .unwrap();
///             // }
///     
///             core::write!(sp_str, "count:{} , FPS:{}", i, fps).unwrap();
///             RoundedRectangle::with_equal_corners(
///                 Rectangle::new(Point::new(0, 40), Size::new(240, 50)),
///                 Size::new(12, 12),
///             )
///             .into_styled(style)
///             .draw(&mut display)
///             .unwrap();
///             Text::with_baseline(
///                 sp_str.as_str(),
///                 Point::new(20, 50),
///                 text_style,
///                 Baseline::Top,
///             )
///             .draw(&mut display)
///             .unwrap();
///             i += size;
///             Timer::after_millis(1000).await
///         }
///     }
///     
///     pub struct NoCsPin;
///     
///     impl ErrorType for NoCsPin {
///         type Error = Infallible;
///     }
///     
///     impl OutputPin for NoCsPin {
///         fn set_low(&mut self) -> Result<(), Self::Error> {
///             Ok(())
///         }
///     
///         fn set_high(&mut self) -> Result<(), Self::Error> {
///             Ok(())
///         }
///     }
///
/// ```
///
pub struct ST7789<SPI, DC, CS, RST, BLK>
where
    SPI: SpiDevice,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    /// SPI interface.
    spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Chip select pin.
    cs: CS,

    /// Reset pin.
    rst: Option<RST>,
    /// Backlight pin.
    blk: Option<BLK>,

    /// Whether the display is RGB (true) or BGR (false).
    _rgb: bool,
    /// Screen Direction Horizontal or vertical
    /// Global image offset.
    pub(crate) width: u32,
    pub(crate) height: u32,
    offset_x: u16,
    offset_y: u16,
    pub(crate) regions: [Option<Region>; 10],
    orientation: Orientation,
}

impl<SPI, DC, CS, RST, BLK> ST7789<SPI, DC, CS, RST, BLK>
where
    SPI: SpiDevice,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    /// Creates a new driver instance that uses hardware SPI.
    ///
    /// # Arguments
    ///
    /// * `spi` - SPI interface.
    /// * `dc` - Data/command pin.
    /// * `rst` - Reset pin.
    /// * `rgb` - Whether the display is RGB (true) or BGR (false).
    /// * `width` - Width of the display.
    /// * `height` - Height of the display.
    pub fn new(
        spi: SPI,
        dc: DC,
        cs: CS,
        rst: Option<RST>,
        blk: Option<BLK>,
        _rgb: bool,
        width: u32,
        height: u32,
    ) -> Self {
        ST7789 {
            spi,
            dc,
            cs,
            rst,
            _rgb,
            width,
            height,
            regions: [None; 10],
            offset_x: 0,
            offset_y: 0,
            blk,
            orientation: Orientation::default(),
        }
    }
    pub fn set_offset(&mut self, offset_x: u16, offset_y: u16) {
        self.offset_x = offset_x;
        self.offset_y = offset_y;
    }
    pub fn set_blk(&mut self, blk: Option<BLK>) {
        self.blk = blk;
    }

    ///
    /// Returns currently set orientation
    ///
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    ///
    /// Sets display orientation
    ///
    pub async fn set_orientation(&mut self, orientation: Orientation) -> Result<(), ()> {
        self.write_command(Commands::MadCtl as u8, &[orientation as u8])
            .await?;
        self.orientation = orientation;

        Ok(())
    }

    /// Initializes the display.
    ///
    /// This function initializes the display by sending a sequence of commands and settings
    /// to configure the display properly. It includes a hardware reset and various configuration
    /// commands.
    ///
    /// # Arguments
    ///
    /// * `delay` - Delay provider.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayNs,
    {
        self.hard_reset(delay)?;
        if let Some(bl) = self.blk.as_mut() {
            bl.set_low().map_err(|_| ())?;
            delay.delay_us(10_000);
            bl.set_high().map_err(|_| ())?;
        }
        // reset display
        self.write_command(Commands::SwReset as u8, &[]).await?;
        delay.delay_us(150_000);
        self.write_command(Commands::SlpOut as u8, &[]).await?; // turn off sleep
        delay.delay_us(10_000);
        self.write_command(Commands::InvOff as u8, &[]).await?; // turn off invert
        self.write_command(Commands::VScrDef as u8, &[]).await?; // vertical scroll definition
        self.write_data(&[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8]).await?; // 0 TSA, 320 VSA, 0 BSA
                                                                    //Orientation
        self.write_command(Commands::MadCtl as u8, &[self.orientation as u8])
            .await?; // left -> right, bottom -> top RGB
        self.write_data(&[0b0000_0000]).await?;
        self.write_command(Commands::ColMod as u8, &[]).await?; // 16bit 65k colors
        self.write_data(&[0b0101_0101]).await?;
        self.write_command(Commands::InvOn as u8, &[]).await?; // hack?
        delay.delay_us(10_000);
        self.write_command(Commands::NorOn as u8, &[]).await?; // turn on display
        delay.delay_us(10_000);
        self.write_command(Commands::DispOn as u8, &[]).await?; // turn on display
        delay.delay_us(10_000);

        // //Set Attributes for Scan Direction
        // if self.sd == VERTICAL {
        //     self.write_command(Instruction::MadCtl as u8, &[0x00])?; // Vertical
        // } else {
        //     self.write_command(Instruction::MadCtl as u8, &[0x78])?; // Horizontal
        // }

        // //Initalize Display
        // //self.write_command(Instruction::MadCtl as u8, &[0x00])?;  //Vertical Screen Direction
        // self.write_command(Instruction::ColMod as u8, &[0x05])?;
        // self.write_command(0xB2, &[0x0B, 0x0B, 0x00, 0x33, 0x35])?;
        // self.write_command(0xB7, &[0x11])?;
        // self.write_command(0xBB, &[0x35])?;
        // self.write_command(0xC0, &[0x2C])?;
        // self.write_command(0xC2, &[0x01])?;
        // self.write_command(0xC3, &[0x0D])?;
        // self.write_command(0xC4, &[0x20])?;
        // self.write_command(0xC6, &[0x13])?;
        // self.write_command(0xD0, &[0xA4, 0xA1])?;
        // self.write_command(0xD6, &[0xA1])?;
        // self.write_command(
        //     0xE0,
        //     &[
        //         0xF0, 0x06, 0x0B, 0x0A, 0x09, 0x26, 0x29, 0x33, 0x41, 0x18, 0x16, 0x15, 0x29, 0x2D,
        //     ],
        // )?;
        // self.write_command(
        //     0xE1,
        //     &[
        //         0xF0, 0x04, 0x08, 0x08, 0x07, 0x03, 0x28, 0x32, 0x40, 0x3B, 0x19, 0x18, 0x2A, 0x2E,
        //     ],
        // )?;
        // self.write_command(0xE4, &[0x25, 0x00, 0x00])?;
        // self.write_command(Instruction::InvOn as u8, &[])?;
        // self.write_command(Instruction::SlpOut as u8, &[])?;

        // delay.delay_ms(120);

        // self.write_command(Instruction::DispOn as u8, &[])?; // Display ON (DISPON)

        Ok(())
    }

    pub fn set_backlight<DELAY>(&mut self, on: bool, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayNs,
    {
        if let Some(bl) = self.blk.as_mut() {
            match on {
                true => bl.set_high().map_err(|_| ())?,
                false => bl.set_low().map_err(|_| ())?,
            }
            delay.delay_us(10); // ensure the pin change will get registered
        }
        Ok(())
    }

    /// Performs a hard reset of the display.
    ///
    /// This function performs a hard reset by toggling the reset pin, ensuring the display
    /// is in a known state before initialization.
    ///
    /// # Arguments
    ///
    /// * `delay` - Delay provider.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub fn hard_reset<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayNs,
    {
        // self.rst.set_high().map_err(|_| ())?;
        // delay.delay_ms(10);
        // self.rst.set_low().map_err(|_| ())?;
        // delay.delay_ms(10);
        // self.rst.set_high().map_err(|_| ())?;
        // delay.delay_ms(10);

        if let Some(rst) = self.rst.as_mut() {
            rst.set_high().map_err(|_| ())?;
            delay.delay_us(10); // ensure the pin change will get registered
            rst.set_low().map_err(|_| ())?;
            delay.delay_us(10); // ensure the pin change will get registered
            rst.set_high().map_err(|_| ())?;
            delay.delay_us(10); // ensure the pin change will get registered
        }

        Ok(())
    }

    /// Writes a command to the display.
    ///
    /// This function sends a command followed by optional parameters to the display.
    ///
    /// # Arguments
    ///
    /// * `command` - Command to write.
    /// * `params` - Parameters for the command.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub(crate) async fn write_command(&mut self, command: u8, params: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_low().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(&[command]).await.map_err(|_| ())?;
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params).await?;
        }
        self.cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    /// Starts data transmission.
    ///
    /// Sets the data/command pin to indicate data mode for subsequent transmissions.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub(crate) fn start_data(&mut self) -> Result<(), ()> {
        self.dc.set_high().map_err(|_| ())
    }

    /// Writes data to the display.
    ///
    /// This function writes data to the display through the SPI interface.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to write.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub(crate) async fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(data).await.map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;
        Ok(())
    }

    /// Writes a data word to the display.
    ///
    /// This function writes a 16-bit word to the display.
    ///
    /// # Arguments
    ///
    /// * `value` - Data word to write.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    async fn write_word(&mut self, value: u16) -> Result<(), ()> {
        self.write_data(&value.to_be_bytes()).await
    }

    /// Sets the address window for the display.
    ///
    /// This function sets the address window for subsequent drawing commands.
    ///
    /// # Arguments
    ///
    /// * `start_x` - Start x-coordinate.
    /// * `start_y` - Start y-coordinate.
    /// * `end_x` - End x-coordinate.
    /// * `end_y` - End y-coordinate.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn set_address_window(
        &mut self,
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    ) -> Result<(), ()> {
        let mut offset = (self.offset_x, self.offset_y);
        match self.orientation {
            Orientation::Portrait | Orientation::PortraitSwapped => {
                offset = (offset.0, offset.1);
            }
            Orientation::Landscape | Orientation::LandscapeSwapped => {
                offset = (offset.1, offset.0);
            }
        }
        let (start_x, start_y, end_x, end_y) = (
            start_x + offset.0,
            start_y + offset.1,
            end_x + offset.0,
            end_y + offset.1,
        );

        self.write_command(Commands::CaSet as u8, &[]).await?;
        self.start_data()?;
        // Write start and end x-coordinates
        self.write_data(&start_x.to_be_bytes()).await?; // Big-endian: splits into two bytes
        self.write_data(&end_x.to_be_bytes()).await?;
        self.write_command(Commands::RaSet as u8, &[]).await?;
        self.start_data()?;
        // Write start and end y-coordinates (with a 20 pixel offset)
        self.write_data(&start_y.to_be_bytes()).await?;
        self.write_data(&end_y.to_be_bytes()).await?;

        self.write_command(0x2C, &[]).await?;

        Ok(())
    }

    /// Clears the screen by filling it with a single color.
    ///
    /// This function sets the entire display to the specified color by writing data
    /// in chunks, which balances memory efficiency and performance.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to fill the screen with, in RGB565 format.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn clear_screen(&mut self, color: u16) -> Result<(), ()> {
        let color_high = (color >> 8) as u8;
        let color_low = (color & 0xff) as u8;

        // Set the address window to cover the entire screen
        self.set_address_window(0, 0, self.width as u16, self.height as u16)
            .await?;
        self.write_command(Commands::RamWr as u8, &[]).await?;
        self.start_data()?;

        // Define a constant for the chunk size
        const CHUNK_SIZE: usize = 1200;
        let mut chunk = [0u8; CHUNK_SIZE * 2];

        // Fill the chunk with the color data
        for i in 0..CHUNK_SIZE {
            chunk[i * 2] = color_high;
            chunk[i * 2 + 1] = color_low;
        }

        // Write data in chunks
        let total_pixels = (self.width * self.height) as usize;
        let full_chunks = total_pixels / CHUNK_SIZE;
        let remaining_pixels = total_pixels % CHUNK_SIZE;

        for _ in 0..full_chunks {
            self.write_data(&chunk).await?;
        }

        if remaining_pixels > 0 {
            self.write_data(&chunk[0..(remaining_pixels * 2)]).await?;
        }

        Ok(())
    }

    // pub fn clear_screen(&mut self, color: u16) -> Result<(), ()> {}
    /// Sets a pixel color at the given coordinates.
    ///
    /// This function sets the color of a single pixel at the specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - X-coordinate.
    /// * `y` - Y-coordinate.
    /// * `color` - Color of the pixel.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn write_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), ()> {
        self.set_address_window(x, y, x, y).await?;
        self.write_command(Commands::RamWr as u8, &[]).await?;
        self.start_data()?;
        self.write_word(color).await
    }

    ///
    /// Sets pixel colors in given rectangle bounds.
    ///
    /// # Arguments
    ///
    /// * `sx` - x coordinate start
    /// * `sy` - y coordinate start
    /// * `ex` - x coordinate end
    /// * `ey` - y coordinate end
    /// * `colors` - anything that can provide `IntoIterator<Item = u16>` to iterate over pixel data
    ///
    pub async fn write_pixels<T>(
        &mut self,
        sx: u16,
        sy: u16,
        ex: u16,
        ey: u16,
        colors: T,
    ) -> Result<(), ()>
    where
        T: IntoIterator<Item = u16>,
    {
        self.set_address_window(sx, sy, ex, ey).await?;
        self.write_command(Commands::RamWr as u8, &[]).await?;
        self.start_data()?;
        for color in colors {
            self.write_word(color).await?;
        }
        Ok(())
    }

    /// Draws an image from a slice of RGB565 data.
    ///
    /// This function draws an image from a slice of pixel data in RGB565 format.
    /// It assumes the image dimensions match the display dimensions.
    ///
    /// # Arguments
    ///
    /// * `image_data` - Image data to draw.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn draw_image(&mut self, image_data: &[u8]) -> Result<(), ()> {
        let width = self.width as u16;
        let height = self.height as u16;

        self.set_address_window(0, 0, width - 1, height - 1).await?;
        self.write_command(Commands::RamWr as u8, &[]).await?;
        self.start_data()?;

        for chunk in image_data.chunks(32) {
            self.write_data(chunk).await?;
        }

        Ok(())
    }

    /// Displays the provided buffer on the screen.
    ///
    /// This function writes the entire buffer to the display, assuming the buffer
    /// contains pixel data for the full display area.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Buffer to display.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success or failure.
    pub async fn show(&mut self, buffer: &[u8]) -> Result<(), ()> {
        self.write_command(Commands::CaSet as u8, &[]).await?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF]).await?;

        self.write_command(Commands::RaSet as u8, &[]).await?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF]).await?;

        self.write_command(Commands::RamWr as u8, &[]).await?;

        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        self.spi.write(buffer).await.map_err(|_| ())?;
        self.cs.set_high().map_err(|_| ())?;

        Ok(())
    }
}

#![no_std]

use core::slice;

use defmt::info;
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::spi::*;
use embedded_hal::blocking::{delay::DelayMs, spi};
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::dma::dma1::{C4, C5};
use stm32f1xx_hal::dma::{TransferPayload, TxDma, WriteDma, *};
use stm32f1xx_hal::pac::SPI2;
use stm32f1xx_hal::spi::{Instance, Spi};

use crate::st7789::cmd::Commands;
use crate::st7789::region::Region;

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
pub const CHUNK_SIZE: usize = 240;
// Spi<Periph<RegisterBlock, 1073756160>, u8>
// TxDma<Spi<Periph<RegisterBlock, 1073756160>, u8>, Ch<Periph<RegisterBlock, 1073872896>, 4>>
// TxDma<Periph<RegisterBlock, 1073756160>, Ch<Periph<RegisterBlock, 1073872896>, 4>>
type SpiTxDma = TxDma<Spi<SPI2, u8>, C5>;
/// Driver for the ST7789 display.
pub struct ST7789<DC, CS, RST, BLK>
where
    // B: embedded_dma::ReadBuffer<Word = u8>,
    // SPI: Instance + TransferPayload,
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    // _marker: core::marker::PhantomData<B>,
    /// SPI interface.
    // spi: SPI,

    /// Data/command pin.
    dc: DC,

    /// Chip select pin.
    cs: CS,

    /// Reset pin.
    rst: Option<RST>,
    /// Backlight pin.
    blk: Option<BLK>,
    dma: Option<SpiTxDma>,
    cmd_buf: Option<&'static mut [u8; 1]>,
    data_buf: Option<&'static mut [u8; 1]>,
    data_2_buf: Option<&'static mut [u8; 2]>,
    data_3_buf: Option<&'static mut [u8; 3]>,
    data_4_buf: Option<&'static mut [u8; 4]>,
    data_8_buf: Option<&'static mut [u8; 8]>,
    //TxDma<Spi<Periph<RegisterBlock, 1073756160>, u8>, Ch<Periph<RegisterBlock, 1073872896>, 4>>
    chunk_buffer: Option<&'static mut [u8; CHUNK_SIZE]>,

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

impl<DC, CS, RST, BLK> ST7789<DC, CS, RST, BLK>
where
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
        dma: SpiTxDma,
        buffer: &'static mut [u8; CHUNK_SIZE],
        cmd_buf: &'static mut [u8; 1],
        data_buf: &'static mut [u8; 1],
        data_2_buf: &'static mut [u8; 2],
        data_3_buf: &'static mut [u8; 3],
        data_4_buf: &'static mut [u8; 4],
        data_8_buf: &'static mut [u8; 8],
        dc: DC,
        cs: CS,
        rst: Option<RST>,
        blk: Option<BLK>,
        _rgb: bool,
        width: u32,
        height: u32,
    ) -> Self {
        ST7789 {
            chunk_buffer: Some(buffer),
            dma: Some(dma),
            cmd_buf: Some(cmd_buf),
            data_buf: Some(data_buf),
            data_2_buf: Some(data_2_buf),
            data_3_buf: Some(data_3_buf),
            data_4_buf: Some(data_4_buf),
            data_8_buf: Some(data_8_buf),
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
    pub fn set_orientation(&mut self, orientation: Orientation) -> Result<(), ()> {
        self.write_command(Commands::MadCtl as u8, &[orientation as u8])?;
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
    pub fn init<DELAY>(&mut self, delay: &mut DELAY) -> Result<(), ()>
    where
        DELAY: DelayUs<u32> + DelayMs<u32>,
    {
        self.hard_reset(delay)?;
        if let Some(bl) = self.blk.as_mut() {
            bl.set_low().map_err(|_| ())?;
            delay.delay_us(10_000);
            bl.set_high().map_err(|_| ())?;
        }
        // reset display
        self.write_command(Commands::SwReset as u8, &[])?;
        delay.delay_us(150_000);
        self.write_command(Commands::SlpOut as u8, &[])?; // turn off sleep
        delay.delay_us(10_000);
        self.write_command(Commands::InvOff as u8, &[])?; // turn off invert
        self.write_command(Commands::VScrDef as u8, &[])?; // vertical scroll definition
        self.write_data(&[0u8, 0u8, 0x14u8, 0u8, 0u8, 0u8])?; // 0 TSA, 320 VSA, 0 BSA
                                                              //Orientation
        self.write_command(Commands::MadCtl as u8, &[self.orientation as u8])?; // left -> right, bottom -> top RGB
        self.write_data(&[0b0000_0000])?;
        self.write_command(Commands::ColMod as u8, &[])?; // 16bit 65k colors
        self.write_data(&[0b0101_0101])?;
        self.write_command(Commands::InvOn as u8, &[])?; // hack?
        delay.delay_us(10_000);
        self.write_command(Commands::NorOn as u8, &[])?; // turn on display
        delay.delay_us(10_000);
        self.write_command(Commands::DispOn as u8, &[])?; // turn on display
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
        DELAY: DelayUs<u32> + DelayMs<u32>,
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
        DELAY: DelayUs<u32> + DelayMs<u32>,
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
    pub(crate) fn write_command(&mut self, command: u8, params: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_low().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        let mut spi_dma = self.dma.take().unwrap();
        let mut cmd_buf = self.cmd_buf.take().unwrap();
        cmd_buf[0] = command;

        let transfer = spi_dma.write(cmd_buf);
        (cmd_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.cmd_buf = Some(cmd_buf);
        if !params.is_empty() {
            self.start_data()?;
            self.write_data(params)?;
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
    pub(crate) fn write_data(&mut self, data: &[u8]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        match data.len() {
            1 => self.write_1_bytes(data)?,
            2 => self.write_2_bytes(data)?,
            3 => self.write_3_bytes(data)?,
            4 => self.write_4_bytes(data)?,
            8 => self.write_8_bytes(data)?,
            _ => Ok(())?,
        };
        if data.len() > 4 && data.len() < 8 {
            self.write_5_7_bytes(data)?;
        }
        if data.len() > 8 {
            let chunck_size = data.len() / 8;

            for i in 0..chunck_size {
                self.write_8_bytes(&data[i * 8..i * 8 + 8])?
            }
            match data.len() {
                1 => self.write_1_bytes(&data[chunck_size * 8..])?,
                2 => self.write_2_bytes(&data[chunck_size * 8..])?,
                3 => self.write_3_bytes(&data[chunck_size * 8..])?,
                4 => self.write_4_bytes(&data[chunck_size * 8..])?,
                _ => self.write_5_7_bytes(&data[chunck_size * 8..])?,
            };
        }

        self.cs.set_high().map_err(|_| ())?;

        Ok(())
    }

    pub(crate) fn write_5_7_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let chunck_size = data.len() / 4;

        for i in 0..chunck_size {
            self.write_4_bytes(&data[i * 4..i * 4 + 4])?
        }
        match data.len() {
            1 => self.write_1_bytes(&data[chunck_size * 4..])?,
            2 => self.write_2_bytes(&data[chunck_size * 4..])?,
            3 => self.write_3_bytes(&data[chunck_size * 4..])?,
            _ => Ok(())?,
        };
        Ok(())
    }

    pub(crate) fn write_1_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_buf.take().unwrap();
        data_buf.copy_from_slice(data);
        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.data_buf = Some(data_buf);
        Ok(())
    }

    pub(crate) fn write_2_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_2_buf.take().unwrap();
        data_buf.copy_from_slice(data);
        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.data_2_buf = Some(data_buf);
        Ok(())
    }

    pub(crate) fn write_3_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_3_buf.take().unwrap();
        data_buf.copy_from_slice(data);
        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.data_3_buf = Some(data_buf);
        Ok(())
    }
    pub(crate) fn write_4_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_4_buf.take().unwrap();
        data_buf.copy_from_slice(data);
        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.data_4_buf = Some(data_buf);
        Ok(())
    }
    pub(crate) fn write_8_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_8_buf.take().unwrap();
        data_buf.copy_from_slice(data);
        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.dma = Some(spi_dma);
        self.data_8_buf = Some(data_buf);
        Ok(())
    }

    pub(crate) fn write_chuck_data(&mut self, data: &[u8; CHUNK_SIZE]) -> Result<(), ()> {
        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        let mut spi_dma = self.dma.take().unwrap();
        let mut chunk_buffer = self.chunk_buffer.take().unwrap();
        chunk_buffer.copy_from_slice(data);

        let transfer = spi_dma.write(chunk_buffer);
        (chunk_buffer, spi_dma) = transfer.wait();

        self.cs.set_high().map_err(|_| ())?;
        self.dma = Some(spi_dma);
        self.chunk_buffer = Some(chunk_buffer);
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
    fn write_word(&mut self, value: u16) -> Result<(), ()> {
        self.write_data(&value.to_be_bytes())
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
    pub fn set_address_window(
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

        self.write_command(Commands::CaSet as u8, &[])?;
        self.start_data()?;
        // Write start and end x-coordinates
        self.write_data(&start_x.to_be_bytes())?; // Big-endian: splits into two bytes
        self.write_data(&end_x.to_be_bytes())?;
        self.write_command(Commands::RaSet as u8, &[])?;
        self.start_data()?;
        // Write start and end y-coordinates (with a 20 pixel offset)
        self.write_data(&start_y.to_be_bytes())?;
        self.write_data(&end_y.to_be_bytes())?;

        self.write_command(0x2C, &[])?;

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
    pub fn clear_screen(&mut self, color: u16) -> Result<(), ()> {
        let color_high = (color >> 8) as u8;
        let color_low = (color & 0xff) as u8;

        // Set the address window to cover the entire screen
        self.set_address_window(0, 0, self.width as u16, self.height as u16)?;
        self.write_command(Commands::RamWr as u8, &[])?;
        self.start_data()?;

        // let mut chunk_buffer = self.chunk_buffer.take().unwrap();
        // let buf_len = chunk_buffer.len();
        // for i in 0..buf_len {
        //     chunk_buffer[i * 2] = color_high;
        //     chunk_buffer[i * 2 + 1] = color_low;
        // }
        // // Write data in chunks
        // let total_pixels = (self.width * self.height) as usize;
        // let full_chunks = total_pixels / CHUNK_SIZE;
        // let remaining_pixels = total_pixels % CHUNK_SIZE;

        // for _ in 0..full_chunks {
        //     self.write_chuck_data(&chunk_buffer)?;
        // }

        // if remaining_pixels > 0 {
        //     self.write_data(&chunk_buffer[0..(remaining_pixels * 2)])?;
        // }
        let chunk_size = CHUNK_SIZE / 2;
        // Define a constant for the chunk size
        let mut chunk = [0u8; CHUNK_SIZE];

        // Fill the chunk with the color data
        for i in 0..chunk_size {
            chunk[i * 2] = color_high;
            chunk[i * 2 + 1] = color_low;
        }

        // Write data in chunks
        let total_pixels = (self.width * self.height) as usize;
        let full_chunks = total_pixels / chunk_size;
        let remaining_pixels = total_pixels % chunk_size;

        for _ in 0..full_chunks {
            // self.write_data(&chunk)?;
            self.write_chuck_data(&chunk)?;
        }

        if remaining_pixels > 0 {
            self.write_data(&chunk[0..(remaining_pixels * 2)])?;
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
    pub fn write_pixel(&mut self, x: u16, y: u16, color: u16) -> Result<(), ()> {
        self.set_address_window(x, y, x, y)?;
        self.write_command(Commands::RamWr as u8, &[])?;
        self.start_data()?;
        self.write_word(color)
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
    pub fn write_pixels<T>(
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
        self.set_address_window(sx, sy, ex, ey)?;
        self.write_command(Commands::RamWr as u8, &[])?;
        self.start_data()?;
        for color in colors {
            self.write_word(color)?;
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
    pub fn draw_image(&mut self, image_data: &[u8]) -> Result<(), ()> {
        let width = self.width as u16;
        let height = self.height as u16;

        self.set_address_window(0, 0, width - 1, height - 1)?;
        self.write_command(Commands::RamWr as u8, &[])?;
        self.start_data()?;

        for chunk in image_data.chunks(32) {
            self.write_data(chunk)?;
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
    pub fn show(&mut self, buffer: &[u8]) -> Result<(), ()> {
        self.write_command(Commands::CaSet as u8, &[])?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF])?;

        self.write_command(Commands::RaSet as u8, &[])?;
        self.write_data(&[0x00, 0x00, 0x00, 0xEF])?;

        self.write_command(Commands::RamWr as u8, &[])?;

        self.cs.set_high().map_err(|_| ())?;
        self.dc.set_high().map_err(|_| ())?;
        self.cs.set_low().map_err(|_| ())?;
        let mut spi_dma = self.dma.take().unwrap();
        let mut data_buf = self.data_buf.take().unwrap();
        data_buf.copy_from_slice(buffer);

        let transfer = spi_dma.write(data_buf);
        (data_buf, spi_dma) = transfer.wait();
        self.cs.set_high().map_err(|_| ())?;
        self.dma.replace(spi_dma);
        self.data_buf.replace(data_buf);
        Ok(())
    }
}

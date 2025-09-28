use crate::st7789::cmd::Commands;
use crate::st7789::st7789::ST7789;
use embedded_hal::blocking::spi::*;
use embedded_hal::digital::v2::OutputPin;

/// Structure to represent a region.
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Region {
    pub x: u16,
    pub y: u16,
    pub width: u32,
    pub height: u32,
}

pub trait RegionExt {
    fn show_regions_and_clear(&mut self, buffer: &[u8]) -> Result<(), ()>;
    fn show_regions(&mut self, buffer: &[u8]) -> Result<(), ()>;
    fn clear_regions(&mut self);
    fn get_regions(&self) -> &[Option<Region>];
    fn store_region_from_params(
        &mut self,
        x: u16,
        y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()>;
    fn store_region(&mut self, region: Region) -> Result<(), ()>;
    fn show_region(
        &mut self,
        buffer: &[u8],
        top_left_x: u16,
        top_left_y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()>;
}

impl<DC, CS, RST, BLK> RegionExt for ST7789<DC, CS, RST, BLK>
where
    DC: OutputPin,
    CS: OutputPin,
    RST: OutputPin,
    BLK: OutputPin,
{
    /// Updates only the specified region of the display with the provided buffer.
    ///
    /// This function updates a specified rectangular region of the display with the pixel data
    /// provided in the buffer. It calculates the necessary offsets and addresses to update only
    /// the designated area, ensuring efficient display refresh.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A slice of bytes representing the pixel data in RGB565 format.
    /// * `top_left_x` - The x-coordinate of the top-left corner of the region to update.
    /// * `top_left_y` - The y-coordinate of the top-left corner of the region to update.
    /// * `width` - The width of the region to update.
    /// * `height` - The height of the region to update.
    ///
    /// # Returns
    ///
    /// `Result<(), ()>` indicating success (`Ok`) or failure (`Err`).
    fn show_region(
        &mut self,
        buffer: &[u8],
        top_left_x: u16,
        top_left_y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()> {
        let start_x = top_left_x as u16; // Start x-coordinate
        let start_y = top_left_y as u16; // Start y-coordinate
        let end_x = (top_left_x as u32 + width - 1) as u16; // End x-coordinate
        let end_y = (top_left_y as u32 + height - 1) as u16; // End y-coordinate

        // Calculate the buffer offset for the region
        let buffer_width = self.width as usize; // Width of the buffer
        let bytes_per_pixel = 2; // Number of bytes per pixel in RGB565 format

        // Set the address window for the region to be updated
        self.set_address_window(start_x, start_y, end_x, end_y)?;

        // Send the command to write to RAM
        self.write_command(Commands::RamWr as u8, &[])?;

        // Start data transmission
        self.start_data()?;

        // Iterate over each row in the region
        for y in start_y..=end_y {
            let start_index = ((y as usize) * buffer_width + (start_x as usize)) * bytes_per_pixel;
            let end_index = start_index + (width as usize) * bytes_per_pixel;

            // Write data to the display in chunks of 32 bytes
            for chunk in buffer[start_index..end_index].chunks(32) {
                self.write_data(chunk)?;
            }
        }

        Ok(())
    }

    fn store_region(&mut self, region: Region) -> Result<(), ()> {
        for i in 0..self.regions.len() {
            if self.regions[i].is_none() {
                self.regions[i] = Some(region);
                return Ok(());
            }
        }
        Err(())
    }

    fn store_region_from_params(
        &mut self,
        x: u16,
        y: u16,
        width: u32,
        height: u32,
    ) -> Result<(), ()> {
        let region = Region {
            x,
            y,
            width,
            height,
        };

        self.store_region(region)
    }

    fn get_regions(&self) -> &[Option<Region>] {
        &self.regions
    }

    fn clear_regions(&mut self) {
        self.regions = [None; 10];
    }

    fn show_regions(&mut self, buffer: &[u8]) -> Result<(), ()> {
        for i in 0..self.regions.len() {
            if self.regions[i].is_some() {
                if let Some(region_data) = self.regions[i] {
                    self.show_region(
                        buffer,
                        region_data.x,
                        region_data.y,
                        region_data.width,
                        region_data.height,
                    )?;
                }
            }
        }

        Ok(())
    }

    // Additional function with default parameter
    fn show_regions_and_clear(&mut self, buffer: &[u8]) -> Result<(), ()> {
        if let Err(e) = self.show_regions(buffer) {
            // Handle the error, e.g., log it or return a different error
            return Err(e);
        }
        self.clear_regions();
        Ok(())
    }
}

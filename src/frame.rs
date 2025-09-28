use embedded_graphics_core::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::{DrawTarget, IntoStorage as _, OriginDimensions, Point, Size},
    Pixel,
};

use crate::region::Region;

/// A structure representing a frame buffer.
pub struct FrameBuffer<'a> {
    buffer: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> FrameBuffer<'a> {
    /// Creates a new frame buffer.
    ///
    /// # Arguments
    ///
    /// * `buffer` - A mutable slice representing the pixel data.
    /// * `width` - The width of the frame buffer.
    /// * `height` - The height of the frame buffer.
    pub fn new(buffer: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    /// Returns a reference to the buffer.
    ///
    /// # Returns
    ///
    /// A reference to the buffer.
    pub fn get_buffer(&self) -> &[u8] {
        self.buffer
    }

    /// Clears the frame buffer with the specified color.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to clear the buffer with.
    pub fn clear(&mut self, color: Rgb565) {
        let raw_color = color.into_storage();
        for chunk in self.buffer.chunks_exact_mut(2) {
            chunk[0] = (raw_color >> 8) as u8;
            chunk[1] = raw_color as u8;
        }
    }

    /// Copies a region from another buffer into this buffer.
    ///
    /// # Arguments
    ///
    /// * `src_buffer` - The source buffer.
    /// * `src_x` - The x-coordinate of the top-left corner of the source region.
    /// * `src_y` - The y-coordinate of the top-left corner of the source region.
    /// * `src_width` - The width of the source region.
    /// * `src_height` - The height of the source region.
    /// * `dest_x` - The x-coordinate of the top-left corner of the destination region.
    /// * `dest_y` - The y-coordinate of the top-left corner of the destination region.
    pub fn copy_region(
        &mut self,
        src_buffer: &[u8],
        src_x: u16,
        src_y: u16,
        src_width: u32,
        src_height: u32,
        dest_x: u16,
        dest_y: u16,
    ) {
        for row in 0..src_height as usize {
            let src_row_start =
                (src_y as usize + row) * self.width as usize * 2 + src_x as usize * 2;
            let src_row_end = src_row_start + src_width as usize * 2;

            let dest_row_start =
                (dest_y as usize + row) * self.width as usize * 2 + dest_x as usize * 2;
            let dest_row_end = dest_row_start + src_width as usize * 2;

            self.buffer[dest_row_start..dest_row_end]
                .copy_from_slice(&src_buffer[src_row_start..src_row_end]);
        }
    }

    /// Restores regions from a source buffer into the frame buffer.
    ///
    /// # Arguments
    ///
    /// * `src_buffer` - The source buffer.
    /// * `regions` - An array of regions to restore.
    pub fn copy_regions(&mut self, src_buffer: &[u8], regions: &[Option<Region>]) {
        for region in regions.iter().flatten() {
            self.copy_region(
                src_buffer,
                region.x,
                region.y,
                region.width,
                region.height,
                region.x,
                region.y,
            );
        }
    }

    /// Compares the current frame buffer with another frame buffer and returns an iterator
    /// of `Pixel` that can be drawn to update the display.
    ///
    /// # Arguments
    ///
    /// * `other` - The other frame buffer to compare against.
    ///
    /// # Returns
    ///
    /// An iterator of `Pixel<Rgb565>`.
    pub fn diff_with<'b>(
        &'b self,
        other: &'b FrameBuffer<'a>,
    ) -> impl Iterator<Item = Pixel<Rgb565>> + 'b {
        self.buffer
            .chunks_exact(2)
            .enumerate()
            .filter_map(move |(i, chunk)| {
                let other_chunk = &other.buffer[i * 2..i * 2 + 2];
                if chunk != other_chunk {
                    let x = (i as u32 % self.width) as i32;
                    let y = (i as u32 / self.width) as i32;
                    let raw_color = u16::from_be_bytes([chunk[0], chunk[1]]);
                    let color = Rgb565::from(RawU16::new(raw_color));
                    Some(Pixel(Point::new(x, y), color))
                } else {
                    None
                }
            })
    }
}

impl<'a> DrawTarget for FrameBuffer<'a> {
    type Color = Rgb565;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0
                && coord.x < self.width as i32
                && coord.y >= 0
                && coord.y < self.height as i32
            {
                let index = ((coord.y as u32 * self.width + coord.x as u32) * 2) as usize;
                let raw_color = color.into_storage();
                self.buffer[index] = (raw_color >> 8) as u8;
                self.buffer[index + 1] = raw_color as u8;
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for FrameBuffer<'a> {
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

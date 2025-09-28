# embassy-st7789v-driver

This is a Rust driver library for ST7789 displays using embassy，embedded_graphics, embedded_hal v1.0, and no_std, no_alloc support.

## Features

These features are enabled by default:

- `graphics` - embedded-graphics support: pulls in [embedded-graphics](https://crates.io/crates/embedded-graphics) dependency
- `batch` - batch-drawing optimization: pulls in [heapless](https://crates.io/crates/heapless) dependency and allocates 300 bytes for frame buffer in the driver
- FrameBuffer
- Region
- embedded-hal 0.2 

# embassy-st7789v-driver  Examples

```rust
#![no_std]
#![no_main]

use core::convert::Infallible;

use defmt::*;
use embassy_executor::Spawner;
use embassy_st7789v_driver::st7789::{Orientation, ST7789};
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    rcc::clocks,
    spi::{self, Config, MisoPin, Spi},
    time::Hertz,
};
use embassy_time::{Delay, Instant, Timer};
use embedded_graphics::pixelcolor::{BinaryColor, Rgb565};
use embedded_graphics::prelude::RgbColor as _;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::primitives::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, RoundedRectangle};
use embedded_graphics::Drawable;
use embedded_graphics::{
    mono_font::ascii::{FONT_6X10, FONT_6X9},
    prelude::DrawTargetExt,
};
use embedded_graphics::{mono_font::MonoTextStyleBuilder, primitives::PrimitiveStyle};
use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, iso_8859_15::FONT_10X20},
    prelude::{IntoStorage, Point, Primitive, Size},
    primitives::{Line, Polyline, Rectangle, Triangle},
    text::{Baseline, Text},
    Pixel,
};
use embedded_graphics_core::draw_target::DrawTarget;
use embedded_hal::digital::{ErrorType, OutputPin};
use embedded_hal_bus::spi::ExclusiveDevice;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
            src: PllSource::HSE,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);
    info!("RCC: {:?}", clocks(&p.RCC));

    let mut spi_config = Config::default();
    spi_config.frequency = Hertz(18_000_000);
    spi_config.mode = spi::Mode {
        polarity: spi::Polarity::IdleLow,
        phase: spi::Phase::CaptureOnFirstTransition,
    };
    spi_config.gpio_speed = Speed::VeryHigh;

    let spi = Spi::new(
        p.SPI2, p.PB13, p.PB15, p.PB14, p.DMA1_CH5, p.DMA1_CH4, spi_config,
    );

    let cs = Output::new(p.PA10, Level::High, Speed::VeryHigh);
    let dc = Output::new(p.PA9, Level::High, Speed::VeryHigh);
    let rst = Output::new(p.PA8, Level::High, Speed::VeryHigh);
    let blk = Output::new(p.PB12, Level::High, Speed::VeryHigh);
    let delay = Delay;
    let spi_device = ExclusiveDevice::new(spi, cs, delay).unwrap();
    // let mut buffer = [0_u8; 512];
    // let di = SpiInterface::new(spi_device, dc, &mut buffer);

    // // create the ILI9486 display driver in rgb666 color mode from the display interface and use a HW reset pin during init
    // let mut display = Builder::new(ST7789, di)
    //     .display_size(135, 240)
    //     .display_offset(52, 40)
    //     .orientation(Orientation::new().rotate(mipidsi::options::Rotation::Deg90))
    //     .invert_colors(mipidsi::options::ColorInversion::Inverted)
    //     .reset_pin(rst)
    //     .init(&mut Delay {})
    //     .unwrap(); // delay provider from your MCU
    //                // clear the display to black
    // display.clear(Rgb565::BLACK).unwrap();

    let mut display = ST7789::new(
        spi_device,
        dc,
        NoCsPin,
        Some(rst),
        Some(blk),
        true,
        240,
        135,
    );
    display
        .set_orientation(Orientation::Landscape)
        .await
        .unwrap();
    display.set_offset(52, 40);
    // initialize
    display.init(&mut Delay {}).await.unwrap();

    info!("init displayer3");
    // display.clear_regions();
    // display.clear_screen(Rgb565::YELLOW.).unwrap();
    // set default orientation
    display.clear(Rgb565::BLACK).unwrap();

    info!("draw");
    let style = PrimitiveStyleBuilder::new()
        .stroke_width(5)
        .stroke_color(Rgb565::GREEN)
        .fill_color(Rgb565::BLACK)
        .build();

    RoundedRectangle::with_equal_corners(
        Rectangle::new(Point::new(14, 16), Size::new(100, 40)),
        Size::new(12, 12),
    )
    .into_styled(style)
    .draw(&mut display)
    .unwrap();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::YELLOW)
        .build();
    Text::with_baseline("Hello", Point::new(43, 26), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    let mut i = 0;
    use core::fmt::Write;
    let line_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();
    // let color = Rgb565::RED;
    // display.set_pixel(12, 12, color).unwrap();

    let mut fps = 0;
    let colors = [
        Rgb565::RED.into_storage(),
        Rgb565::GREEN.into_storage(),
        Rgb565::BLUE.into_storage(),
        Rgb565::YELLOW.into_storage(),
        Rgb565::MAGENTA.into_storage(),
        Rgb565::CYAN.into_storage(),
    ];
    // let colors = [
    //     Rgb565::RED,
    //     Rgb565::GREEN,
    //     Rgb565::BLUE,
    //     Rgb565::YELLOW,
    //     Rgb565::MAGENTA,
    //     Rgb565::CYAN,
    // ];
    let size = 1000;
    let mut sp_str = String::<24>::new();
    loop {
        let now = Instant::now();
        for j in 0..size {
            display
                .clear_screen(colors[j % colors.len()])
                .await
                .unwrap();
            // display.clear(colors[j % colors.len()]).unwrap();
        }
        fps = 1000 * size as usize / now.elapsed().as_millis() as usize;
        info!("FPS:{}", fps);
        sp_str.clear();

        // for y in 0..135 {
        //     display
        //         .set_pixels(0, y, 240, 135, [Rgb565::BLUE; 240].into_iter())
        //         .unwrap();
        // }

        core::write!(sp_str, "count:{} , FPS:{}", i, fps).unwrap();
        RoundedRectangle::with_equal_corners(
            Rectangle::new(Point::new(0, 40), Size::new(240, 50)),
            Size::new(12, 12),
        )
        .into_styled(style)
        .draw(&mut display)
        .unwrap();
        Text::with_baseline(
            sp_str.as_str(),
            Point::new(20, 50),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();
        i += size;
        Timer::after_millis(1000).await
    }
}

pub struct NoCsPin;

impl ErrorType for NoCsPin {
    type Error = Infallible;
}

impl OutputPin for NoCsPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

```

dma max

![](C:\Users\hy\AppData\Roaming\marktext\images\2025-09-28-21-13-29-image.png)
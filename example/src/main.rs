#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::info;
use st7789v_driver::st7789::{Orientation, ST7789};
// use defmt_rtt as _;
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_6X9};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor as _;
use embedded_graphics::primitives::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, RoundedRectangle};
use embedded_graphics::Drawable;
use embedded_graphics::{mono_font::MonoTextStyleBuilder, primitives::PrimitiveStyle};
use embedded_graphics::{
    mono_font::{ascii::FONT_9X15, iso_8859_15::FONT_10X20},
    prelude::{IntoStorage, Point, Primitive, Size},
    primitives::{Line, Polyline, Rectangle, Triangle},
    text::{Baseline, Text},
    Pixel,
};
use embedded_graphics_core::{draw_target::DrawTarget, prelude::RgbColor};
use heapless::String;
// use panic_semihosting as _;
use stm32f1xx_hal::pac::adc3::sqr1::R;
use stm32f1xx_hal::pac::{self, SPI2};
use stm32f1xx_hal::rcc;
use stm32f1xx_hal::rtc::Rtc;
use stm32f1xx_hal::spi::{Mode, Phase, Polarity};
use stm32f1xx_hal::{prelude::*, time::MonoTimer};
use {defmt_rtt as _, panic_probe as _};
#[entry]
fn main() -> ! {
    //初始化和获取外设对象
    // 获取cortex-m 相关的核心外设
    let cp = cortex_m::Peripherals::take().unwrap();
    //获取stm32f1xx_hal硬件外设
    let dp = pac::Peripherals::take().unwrap();
    // 初始化并获取flash和rcc设备的所有权
    let mut flash = dp.FLASH.constrain();
    //冻结系统中所有时钟的配置，并将冻结后的频率值存储在“clocks”中
    // let clocks = rcc.cfgr.freeze(&mut flash.acr);
    info!("init");
    let sysclk = 72.MHz();
    let pclk = sysclk / 2;
    let mut rcc = dp.RCC.freeze(
        rcc::Config::hse(8.MHz())
            .sysclk(sysclk)
            .pclk1(pclk)
            .pclk2(sysclk)
            .hclk(sysclk),
        &mut flash.acr,
    );
    let clocks = rcc.clocks;
    // dp.TIM3.monotonic_us(&mut rcc);
    info!(
        "sysclk:{:?},pclk1:{:?},pclk2:{:?},hclk:{:?},adcclk:{:?}",
        clocks.sysclk().to_Hz(),
        clocks.pclk1().to_Hz(),
        clocks.pclk2().to_Hz(),
        clocks.hclk().to_Hz(),
        clocks.adcclk().to_Hz()
    );
    let mut delay = dp.TIM3.delay_us(&mut rcc);
    let mut afio = dp.AFIO.constrain(&mut rcc);
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    let mut display = {
        let pins = (Some(gpiob.pb13), SPI2::NoMiso, Some(gpiob.pb15));
        let spi_mode = Mode {
            polarity: Polarity::IdleLow,
            phase: Phase::CaptureOnFirstTransition,
        };
        let spi = dp.SPI2.spi(pins, spi_mode, 18.MHz(), &mut rcc);
        // Set up the DMA device
        // let dma = dp.DMA1.split(&mut rcc);

        // Connect the SPI device to the DMA
        // let spi = spi.with_rx_tx_dma(dma.4, dma.5);
        // let spi = spi.with_rx_dma(dma.4);
        // let spi_dma = spi.with_tx_dma(dma.5);

        // let mut buf = [0u8; 12];
        // let transfer = spi_dma.write(&buf);
        // let (_buffer, _spi_dma) = transfer.wait();
        // let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
        info!("init displayer1");
        let dc = gpioa.pa9.into_push_pull_output(&mut gpioa.crh);
        let cs = gpioa.pa10.into_push_pull_output(&mut gpioa.crh);
        let rst = gpioa.pa8.into_push_pull_output(&mut gpioa.crh);
        let blk = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
        // blk.set_high();
        info!("init displayer2");
        let mut display = ST7789::new(spi, dc, cs, Some(rst), Some(blk), true, 240, 135);
        display.set_orientation(Orientation::Landscape).unwrap();
        display.set_offset(52, 40);
        // initialize
        display.init(&mut delay).unwrap();

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
        display
    };
    let mut i = 0;
    use core::fmt::Write;
    let line_style = PrimitiveStyle::with_stroke(Rgb565::GREEN, 1);
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(Rgb565::WHITE)
        .build();
    // let mut pwr = dp.PWR;
    // let mut bkp = dp.BKP.constrain(&mut pwr, &mut rcc);
    // let rtc = Rtc::new(dp.RTC, &mut bkp, &mut rcc);
    let mono = MonoTimer::new(cp.DWT, cp.DCB, &clocks);
    let frequency = mono.frequency();
    let colors = [
        Rgb565::RED,
        Rgb565::GREEN,
        Rgb565::BLUE,
        Rgb565::YELLOW,
        Rgb565::MAGENTA,
        Rgb565::CYAN,
    ];
    let size = 200;
    let mut sp_str = String::<24>::new();
    let mut fps = 0;
    loop {
        let now = mono.now();
        for j in 0..size {
            display
                .clear_screen(colors[j % colors.len()].into_storage())
                .unwrap();
            // display.clear(colors[j % colors.len()]).unwrap();
        }
        fps = size as usize / (now.elapsed() as usize / frequency.to_Hz() as usize);
        info!("FPS:{}", fps);
        sp_str.clear();
        core::write!(sp_str, "count:{} , FPS:{}", i, fps).unwrap();
        Text::with_baseline(sp_str.as_str(), Point::new(0, 0), text_style, Baseline::Top)
            .draw(&mut display)
            .unwrap();

        i += size;
        delay.delay_ms(1000_u16);
    }
}

#![no_std]
#![no_main]
#![feature(asm)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_rtt as _;
use panic_probe as _;

use embassy::executor::Spawner;
use embassy::time::{Delay, Duration, Timer};

use embassy_rp::gpio::NoPin;
use embassy_rp::spi;
use embassy_rp::spi::Spi;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    Peripherals,
};

use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use epd_waveshare::prelude::*;

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    pixelcolor::BinaryColor::{Off, On},
    prelude::*,
    primitives::*,
    text::{Baseline, Text, TextStyleBuilder},
};

use embedded_text::{alignment::VerticalAlignment, TextBox};

defmt::timestamp! {"{=u64}", {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mosi = p.PIN_3;
    let clk = p.PIN_2;

    let cs = Output::new(p.PIN_5, Level::Low);
    let dc = Output::new(p.PIN_20, Level::Low);
    let rst = Output::new(p.PIN_21, Level::Low);
    let busy = Input::new(p.PIN_22, Pull::None);

    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut config = spi::Config::default();
    config.frequency = 2_000_000;
    let mut spi = Spi::new(p.SPI0, clk, mosi, NoPin, NoPin, config);

    let mut delay = Delay;
    let mut epd = Epd2in9::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    let mut display = Display2in9::default();

    let _ = display.clear(Off);

    let _ = epd.update_old_frame(&mut spi, display.buffer(), &mut delay);
    let _ = epd.display_frame(&mut spi, &mut delay);

    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X12)
        .text_color(On)
        .background_color(Off)
        .build();

    let _ = TextBox::with_vertical_alignment(
        "Hello from Rust on the RP2040, connected to an E-Ink display...",
        Rectangle::new(
            Point::new(1, 1),
            Size::new(epd.width() - 2, epd.height() - 2),
        ),
        style,
        VerticalAlignment::Scrolling,
    )
    .draw(&mut display);

    let _ = epd.update_and_display_new_frame(&mut spi, display.buffer(), &mut delay);

    loop {
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;

        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[allow(unused)]
fn draw_text(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X12)
        .text_color(On)
        .background_color(Off)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

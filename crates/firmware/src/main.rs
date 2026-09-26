#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    i2c::master::{Config as I2cConfig, I2c},
    main,
    time::Rate,
};
use esp_println::println;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

use enigma::EnigmaSettings;

const KAT_INPUT: &str = "AAAAAAAAAAAAAAAAAAAAAAAAA";
const KAT_EXPECTED: &str = "BDZGOWCXLTKSBTMCDLPBMUQOF";
const PLAINTEXT: &str = "HELLOWORLD";

fn bdzgo_settings() -> EnigmaSettings {
    EnigmaSettings {
        rotors: [String::from("I"), String::from("II"), String::from("III")],
        reflector: String::from("B"),
        rings: String::from("AAA"),
        positions: String::from("AAA"),
        plugboard: String::new(),
    }
}

#[main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let i2c = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)),
    )
    .unwrap()
    .with_sda(peripherals.GPIO5)
    .with_scl(peripherals.GPIO6);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let settings = bdzgo_settings();
    let kat = enigma::encrypt(&settings, KAT_INPUT).unwrap();
    let kat_ok = kat == KAT_EXPECTED;
    let cipher = enigma::encrypt(&settings, PLAINTEXT).unwrap();

    println!("Enigma M3 (I/II/III, refl B, rings AAA, pos AAA)");
    println!("KAT {}: {}", if kat_ok { "PASS" } else { "FAIL" }, kat);
    println!("{PLAINTEXT} -> {cipher}");

    Text::with_baseline("Enigma M3", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    Text::with_baseline(PLAINTEXT, Point::new(0, 14), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    Text::with_baseline(&cipher, Point::new(0, 26), text_style, Baseline::Top)
        .draw(&mut display)
        .unwrap();
    Text::with_baseline(
        if kat_ok { "KAT: PASS" } else { "KAT: FAIL" },
        Point::new(0, 50),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();
    display.flush().unwrap();

    let delay = Delay::new();
    loop {
        delay.delay_millis(1000);
    }
}

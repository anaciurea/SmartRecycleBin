#![no_main]
#![no_std]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_time::{Delay, Timer};
use mfrc522::Mfrc522;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Sistem pornit!");

    let mut rfid_rst = Output::new(p.PA3, Level::High, Speed::Low);
    let rfid_cs = Output::new(p.PA8, Level::High, Speed::Low);

    let spi = Spi::new_blocking(
        p.SPI1,
        p.PA5, // SCK
        p.PA7, // MOSI
        p.PA6, // MISO
        SpiConfig::default(),
    );

    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new(spi, rfid_cs, Delay);
    let spi_itf = mfrc522::comm::blocking::spi::SpiInterface::new(spi_device);
    let rfid = Mfrc522::new(spi_itf);

    rfid_rst.set_low();
    Timer::after_millis(50).await;
    rfid_rst.set_high();
    Timer::after_millis(50).await;

    let mut rfid = rfid.init().unwrap();

    info!("RFID Initializat. Astept carduri...");

    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                let id = uid.as_bytes();
                info!("Card detectat! UID: {=[u8]:x}", id);
                Timer::after_secs(2).await;
            }
        }
        Timer::after_millis(100).await;
    }
}

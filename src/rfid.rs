#![no_main]
#![no_std]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::flash::{Blocking, Flash};
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_time::{Delay, Timer};
use mfrc522::Mfrc522;
use {defmt_rtt as _, panic_probe as _};

// Ultima pagina de flash (8KB): offset 504K de la inceputul flash-ului
const STORAGE_OFFSET: u32 = 504 * 1024;
// Valoare magica pentru a verifica daca datele sunt valide
const MAGIC: u32 = 0xAB_CD_12_34;

fn read_points(flash: &mut Flash<Blocking>) -> u32 {
    let mut buf = [0u8; 16];
    flash.blocking_read(STORAGE_OFFSET, &mut buf).ok();
    let magic = u32::from_le_bytes(buf[0..4].try_into().unwrap());
    if magic == MAGIC {
        u32::from_le_bytes(buf[4..8].try_into().unwrap())
    } else {
        0
    }
}

fn save_points(flash: &mut Flash<Blocking>, points: u32) {
    flash.blocking_erase(STORAGE_OFFSET, STORAGE_OFFSET + 8 * 1024).ok();
    let mut buf = [0xFFu8; 16];
    buf[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    buf[4..8].copy_from_slice(&points.to_le_bytes());
    flash.blocking_write(STORAGE_OFFSET, &buf).ok();
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Sistem pornit!");

    let mut flash = Flash::new_blocking(p.FLASH);

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

    let mut puncte = read_points(&mut flash);
    info!("RFID Initializat. Puncte salvate: {}", puncte);

    loop {
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                let id = uid.as_bytes();
                puncte += 200;
                info!("Card detectat! UID: {=[u8]:x}", id);
                info!("Felicitari! Ai reciclat cu succes!");
                info!("Ai primit 200 de puncte. Total puncte: {}", puncte);
                save_points(&mut flash, puncte);
                Timer::after_secs(2).await;
            }
        }
        Timer::after_millis(100).await;
    }
}

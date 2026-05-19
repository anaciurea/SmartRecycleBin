#![no_std]
#![no_main]

mod buzzer;
mod servo;

use buzzer::Buzzer;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::flash::{Blocking, Flash};
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed, OutputType};
use embassy_stm32::peripherals::{FLASH, PA3, PA5, PA6, PA7, PA8, PB5, PB10, PC7, PC8, SPI1, TIM2, TIM3};
use embassy_stm32::spi::{Config as SpiConfig, Spi};
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{low_level::CountingMode, Channel};
use embassy_stm32::Peri;
use embassy_time::{Delay, Instant, Timer};
use mfrc522::Mfrc522;
use servo::Servo;
use {defmt_rtt as _, panic_probe as _};

// --- Flash (memorie persistenta pentru puncte RFID) ---
const STORAGE_OFFSET: u32 = 504 * 1024;
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

// --- RFID (RC522) --- SPI1: SCK=PA5, MOSI=PA7, MISO=PA6, CS=PA8, RST=PA3 ---
#[embassy_executor::task]
async fn rfid_task(
    flash_periph: Peri<'static, FLASH>,
    spi: Peri<'static, SPI1>,
    sck: Peri<'static, PA5>,
    mosi: Peri<'static, PA7>,
    miso: Peri<'static, PA6>,
    cs: Peri<'static, PA8>,
    rst: Peri<'static, PA3>,
) {
    // Flash - stocare puncte
    let mut flash = Flash::new_blocking(flash_periph);

    // RFID - initializare SPI si reset
    let mut rfid_rst = Output::new(rst, Level::High, Speed::Low);
    let rfid_cs = Output::new(cs, Level::High, Speed::Low);
    let spi = Spi::new_blocking(spi, sck, mosi, miso, SpiConfig::default());
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new(spi, rfid_cs, Delay);
    let spi_itf = mfrc522::comm::blocking::spi::SpiInterface::new(spi_device);
    let rfid = Mfrc522::new(spi_itf);

    rfid_rst.set_low();
    Timer::after_millis(50).await;
    rfid_rst.set_high();
    Timer::after_millis(50).await;

    let mut rfid = rfid.init().unwrap();

    // Flash - citire puncte salvate la pornire
    let mut puncte = read_points(&mut flash);
    info!("RFID initializat. Puncte salvate: {}", puncte);

    loop {
        // RFID - detectare card
        if let Ok(atqa) = rfid.reqa() {
            if let Ok(uid) = rfid.select(&atqa) {
                let id = uid.as_bytes();
                puncte += 200;
                info!("--- RFID ---");
                info!("Card detectat! UID: {=[u8]:x}", id);
                info!("Felicitari! Ai reciclat cu succes!");
                info!("Ai primit 200 de puncte. Total puncte: {}", puncte);

                // Flash - salvare puncte dupa fiecare scanare
                save_points(&mut flash, puncte);
                Timer::after_secs(2).await;
            }
        }
        Timer::after_millis(100).await;
    }
}

// --- HC-SR04 (senzor ultrasonic) --- TRIG=PC7, ECHO=PC8 ---
// --- Servo (SG90)               --- TIM3 CH2, semnal=PB5  ---
// --- Buzzer (KY-006)            --- TIM2 CH3, semnal=PB10 ---
#[embassy_executor::task]
async fn hc_task(
    trig_pin: Peri<'static, PC7>,
    echo_pin: Peri<'static, PC8>,
    tim3: Peri<'static, TIM3>,
    pb5: Peri<'static, PB5>,
    tim2: Peri<'static, TIM2>,
    pb10: Peri<'static, PB10>,
) {
    // HC-SR04 - pini trig si echo
    let mut trig = Output::new(trig_pin, Level::Low, Speed::VeryHigh);
    let echo = Input::new(echo_pin, Pull::None);

    // Servo - TIM3 CH2 pe PB5, frecventa 50Hz
    let ch2 = PwmPin::new(pb5, OutputType::PushPull);
    let pwm_servo = SimplePwm::new(tim3, None, Some(ch2), None, None, hz(50), CountingMode::EdgeAlignedUp);
    let mut servo = Servo::new(pwm_servo, Channel::Ch2);

    // Buzzer - TIM2 CH3 pe PB10, frecventa 2kHz
    let ch3 = PwmPin::new(pb10, OutputType::PushPull);
    let pwm_buzzer = SimplePwm::new(tim2, None, None, Some(ch3), None, hz(2000), CountingMode::EdgeAlignedUp);
    let mut buzzer = Buzzer::new(pwm_buzzer, Channel::Ch3);

    // Servo - pozitie initiala 0 grade
    servo.set_angle(-90);

    info!("HC-SR04 initializat. Masor distanta...");

    loop {
        // HC-SR04 - trimitere puls de 10µs
        trig.set_high();
        Timer::after_micros(10).await;
        trig.set_low();

        // HC-SR04 - masurare durata echo
        while echo.is_low() {}
        let start_time = Instant::now();
        while echo.is_high() {
            if start_time.elapsed().as_micros() > 30000 {
                break;
            }
        }
        let duration = Instant::now().duration_since(start_time).as_micros();

        if duration < 30000 {
            let distance_cm = duration / 58;
            info!("--- HC-SR04 --- Distanta: {} cm", distance_cm);

            if distance_cm < 30 {
                info!("Mana detectata la {} cm! Servo -> 0 grade.", distance_cm);
                servo.set_angle(-90); // -90 in API = 0 grade fizic
                buzzer.beep(200).await;
            } else {
                servo.set_angle(0); // 0 in API = 90 grade fizic
            }
        } else {
            info!("--- HC-SR04 --- Nu vad niciun obstacol in apropiere.");
            servo.set_angle(0);
        }

        Timer::after_millis(500).await;
    }
}

// --- Entry point ---
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Sistem pornit!");

    // Pornire task RFID + Flash
    spawner
        .spawn(rfid_task(p.FLASH, p.SPI1, p.PA5, p.PA7, p.PA6, p.PA8, p.PA3))
        .unwrap();

    // Pornire task HC-SR04 + Servo + Buzzer
    spawner
        .spawn(hc_task(p.PC7, p.PC8, p.TIM3, p.PB5, p.TIM2, p.PB10))
        .unwrap();
}

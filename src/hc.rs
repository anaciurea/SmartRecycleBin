#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initializam perifericele placii
    let p = embassy_stm32::init(Default::default());

    // PC7 -> trig
    let mut trig = Output::new(p.PC7, Level::Low, Speed::VeryHigh);

    // echo -> PC8 (3.3V)
    let echo = Input::new(p.PC8, Pull::None);

    info!("Sensor working...");

    loop {
        // puls de initiere
        trig.set_high();
        Timer::after_micros(10).await;
        trig.set_low();

        // asteptam echo sa devina 1
        while echo.is_low() {}
        let start_time = Instant::now();

        // asteptam sa se intoarca ecoul, adica echo sa devina 0
        while echo.is_high() {
            if start_time.elapsed().as_micros() > 30000 {
                break;
            }
        }
        let end_time = Instant::now();

        // timp total pentru dus-intors al ecoului
        let duration = end_time.duration_since(start_time).as_micros();

        if duration < 30000 {
            let distance_cm = duration / 58;
            info!("Distanta masurata: {} cm", distance_cm);
        } else {
            info!("Nu vad niciun obstacol in apropriere.");
        }

        Timer::after_millis(500).await;
    }
}

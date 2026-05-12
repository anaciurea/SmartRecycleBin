#![no_std]
#![no_main]

mod buzzer;
mod servo;

use buzzer::Buzzer;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::time::hz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{low_level::CountingMode, Channel};
use embassy_time::Timer;
use servo::Servo;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // PB5 -> semnal servo (TIM3 CH2)
    let ch2 = PwmPin::new(p.PB5, OutputType::PushPull);
    let pwm_servo = SimplePwm::new(
        p.TIM3,
        None,
        Some(ch2),
        None,
        None,
        hz(50),
        CountingMode::EdgeAlignedUp,
    );
    let mut servo = Servo::new(pwm_servo, Channel::Ch2);

    // PB10 -> semnal buzzer (TIM2 CH3)
    // Conectare: S -> PB10, mijloc -> 3.3V, - -> GND
    let ch3 = PwmPin::new(p.PB10, OutputType::PushPull);
    let pwm_buzzer = SimplePwm::new(
        p.TIM2,
        None,
        None,
        Some(ch3),
        None,
        hz(2000), // 2kHz - in intervalul optim al KY-006
        CountingMode::EdgeAlignedUp,
    );
    let mut buzzer = Buzzer::new(pwm_buzzer, Channel::Ch3);

    info!("Servo + Buzzer pornite!");

    loop {
        // Sweep -90 -> +90, buzzer pornit
        buzzer.on();
        for angle in (-90..=90).step_by(1) {
            servo.set_angle(angle);
            Timer::after_micros(500).await;
        }

        // Beep scurt la capat
        buzzer.off();
        Timer::after_millis(100).await;
        buzzer.beep(100).await;

        // Sweep +90 -> -90, buzzer pornit
        buzzer.on();
        for angle in (-90..=90).rev().step_by(1) {
            servo.set_angle(angle);
            Timer::after_micros(500).await;
        }

        // Beep scurt la capat
        buzzer.off();
        Timer::after_millis(100).await;
        buzzer.beep(100).await;
    }
}

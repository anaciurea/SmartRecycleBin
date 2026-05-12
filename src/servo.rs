#![allow(dead_code)]

use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{Channel, GeneralInstance4Channel};
use embedded_hal::Pwm;

const PULSE_MIN_US: u32 = 500;
const PULSE_MAX_US: u32 = 2500;
const PERIOD_US: u32 = 20_000;

pub struct Servo<'d, T: GeneralInstance4Channel> {
    pwm: SimplePwm<'d, T>,
    channel: Channel,
}

impl<'d, T: GeneralInstance4Channel> Servo<'d, T> {
    pub fn new(pwm: SimplePwm<'d, T>, channel: Channel) -> Self {
        let mut s = Self { pwm, channel };
        s.pwm.enable(channel);
        s
    }

    pub fn set_angle(&mut self, angle: i32) {
        let angle = angle.clamp(-90, 90);
        let pulse_us =
            (((angle + 90) as u32) * (PULSE_MAX_US - PULSE_MIN_US) / 180) + PULSE_MIN_US;
        let max = self.pwm.get_max_duty() as u32;
        let duty = (pulse_us * max) / PERIOD_US;
        self.pwm.set_duty(self.channel, duty);
    }

    pub fn set_pulse_us(&mut self, pulse_us: u32) {
        let pulse_us = pulse_us.clamp(PULSE_MIN_US, PULSE_MAX_US);
        let max = self.pwm.get_max_duty() as u32;
        let duty = (pulse_us * max) / PERIOD_US;
        self.pwm.set_duty(self.channel, duty);
    }

    pub fn disable(&mut self) {
        self.pwm.disable(self.channel);
    }
}

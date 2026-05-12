#![allow(dead_code)]

use embassy_stm32::timer::simple_pwm::SimplePwm;
use embassy_stm32::timer::{Channel, GeneralInstance4Channel};
use embassy_time::Timer;
use embedded_hal::Pwm;

pub struct Buzzer<'d, T: GeneralInstance4Channel> {
    pwm: SimplePwm<'d, T>,
    channel: Channel,
}

impl<'d, T: GeneralInstance4Channel> Buzzer<'d, T> {
    pub fn new(pwm: SimplePwm<'d, T>, channel: Channel) -> Self {
        Self { pwm, channel }
    }

    pub fn on(&mut self) {
        let max = self.pwm.get_max_duty();
        self.pwm.set_duty(self.channel, max / 2);
        self.pwm.enable(self.channel);
    }

    pub fn off(&mut self) {
        self.pwm.disable(self.channel);
    }

    pub async fn beep(&mut self, duration_ms: u64) {
        self.on();
        Timer::after_millis(duration_ms).await;
        self.off();
    }
}

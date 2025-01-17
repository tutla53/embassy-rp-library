//! Servo PWM Builder with PIO
#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use {
    core::time::Duration,
    embassy_rp::{
        pio::Instance,
        pio_programs::pwm::PioPwm,
    },
    {defmt_rtt as _, panic_probe as _},
};

const DEFAULT_MIN_PULSE_WIDTH: u64 = 1000; // uncalibrated default, the shortest duty cycle sent to a servo
const DEFAULT_MAX_PULSE_WIDTH: u64 = 2000; // uncalibrated default, the longest duty cycle sent to a servo
const DEFAULT_MAX_DEGREE_ROTATION: u64 = 180; // 180 degrees is typical
const DEFAULT_INITIAL_POSITION: u64 = 0; // 180 degrees is typical
const REFRESH_INTERVAL: u64 = 20000; // The period of each cycle

pub struct ServoPioBuilder<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
    period: Duration,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    max_degree_rotation: u64,
    initial_position: u64,
}

impl<'d, T: Instance, const SM: usize> ServoPioBuilder<'d, T, SM> {
    pub fn new(pwm: PioPwm<'d, T, SM>) -> Self {
        Self {
            pwm,
            period: Duration::from_micros(REFRESH_INTERVAL),
            min_pulse_width: Duration::from_micros(DEFAULT_MIN_PULSE_WIDTH),
            max_pulse_width: Duration::from_micros(DEFAULT_MAX_PULSE_WIDTH),
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
            initial_position: DEFAULT_INITIAL_POSITION,
        }
    }

    pub fn set_period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    pub fn set_min_pulse_width(mut self, duration: Duration) -> Self {
        self.min_pulse_width = duration;
        self
    }

    pub fn set_max_pulse_width(mut self, duration: Duration) -> Self {
        self.max_pulse_width = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u64) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn set_initial_position(mut self, pos: u64) -> Self {
        self.initial_position = pos;
        self
    }

    pub fn build(mut self) -> Servo<'d, T, SM> {
        self.pwm.set_period(self.period);
        Servo {
            pwm: self.pwm,
            min_pulse_width: self.min_pulse_width,
            max_pulse_width: self.max_pulse_width,
            max_degree_rotation: self.max_degree_rotation,
            current_pos: self.initial_position,
        }
    }
}

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,
    min_pulse_width: Duration,
    max_pulse_width: Duration,
    max_degree_rotation: u64,
    current_pos: u64,
}

#[allow(dead_code)]
impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    fn set_current_pos(&mut self, degree: u64){
        self.current_pos = degree
    }

    pub fn get_current_pos(&mut self) -> u64 {
        return self.current_pos
    }

    pub fn start(&mut self) {
        self.pwm.start();
        self.rotate(self.current_pos);
    }

    pub fn stop(&mut self) {
        self.pwm.stop();
    }

    pub fn write_time(&mut self, duration: Duration) {
        self.pwm.write(duration);
    }

    pub fn rotate(&mut self, degree: u64) {
        let degree_per_nano_second = (self.max_pulse_width.as_nanos() as u64 - self.min_pulse_width.as_nanos() as u64)
            / self.max_degree_rotation;
        let mut duration =
            Duration::from_nanos(degree * degree_per_nano_second + self.min_pulse_width.as_nanos() as u64);
        if self.max_pulse_width < duration {
            duration = self.max_pulse_width;
        }
        self.set_current_pos(degree);
        self.write_time(duration);
    }
}
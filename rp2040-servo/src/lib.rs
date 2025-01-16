//! Servo PWM Builder
#![no_std]
#![no_main]
#![allow(async_fn_in_trait)]

use {
    embassy_rp::pwm::{
        Config, 
        Pwm, 
        SetDutyCycle
    },
    {defmt_rtt as _, panic_probe as _},
};

const DEFAULT_SERVO_FREQ: u32 = 50; //Hertz
const DEFAULT_MIN_DUTY: u32 = 2100; //us 
const DEFAULT_MAX_DUTY: u32 = 8200;  //us
const DEFAULT_MAX_DEGREE_ROTATION: u32 = 180; //degree
const DEFAULT_INITIAL_POSITION: u32 = 0; //degree

pub struct ServoBuilder<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    freq: u32, 
    min_duty: u32,
    max_duty: u32,
    max_degree_rotation: u32,
    initial_position: u32,
}

impl<'d> ServoBuilder<'d> {
    pub fn new(pwm: Pwm<'d>) -> Self {
        Self {
            pwm,
            cfg: Config::default(),
            freq: DEFAULT_SERVO_FREQ,
            min_duty: DEFAULT_MIN_DUTY,
            max_duty: DEFAULT_MAX_DUTY,
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
            initial_position: DEFAULT_INITIAL_POSITION,
        }
    }

    pub fn set_servo_freq(mut self, freq: u32) -> Self {
        self.freq = freq;
        self
    }

    pub fn set_min_duty(mut self, duration: u32) -> Self {
        self.min_duty = duration;
        self
    }

    pub fn set_max_duty(mut self, duration: u32) -> Self {
        self.max_duty = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u32) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn set_initial_position(mut self, init_pos: u32) -> Self {
        self.initial_position = init_pos;
        self
    }

    pub fn build(mut self) -> Servo<'d> {
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 40u8;
        let period = (clock_freq_hz / (self.freq * divider as u32)) as u16 - 1;
        
        self.cfg.top = period;
        self.cfg.divider = divider.into();
        self.pwm.set_config(&self.cfg);

        Servo {
            pwm: self.pwm,
            cfg: self.cfg,
            min_duty: self.min_duty,
            max_duty: self.max_duty,
            max_degree_rotation:  self.max_degree_rotation,
            current_pos: self.initial_position,
        }
    }
}

pub struct Servo<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    min_duty: u32,
    max_duty: u32,
    max_degree_rotation: u32,
    current_pos: u32,
}

#[allow(dead_code)]
impl<'d> Servo<'d> {
    fn set_current_pos(&mut self, degree: u32){
        self.current_pos = degree;
    }

    pub fn get_current_pos(&mut self) -> u32 {
        return self.current_pos;
    }

    pub fn get_min_duty(&mut self) -> u32 {
        return self.min_duty;
    }

    pub fn get_max_duty(&mut self) -> u32 {
        return self.max_duty;
    }

    pub fn get_current_duty(&mut self) -> u32 {
        let duty = self.degree_to_duty(self.current_pos);
        return duty;
    }

    pub fn enable(&mut self) {
        self.cfg.enable = true;
        self.pwm.set_config(&self.cfg);
        self.rotate(self.current_pos);
    }

    pub fn disable(&mut self) {
        self.cfg.enable = false;
        self.pwm.set_config(&self.cfg);
    }

    pub fn degree_to_duty(&mut self, degree: u32) -> u32 {
        let mut duty = (((self.max_duty - self.min_duty) * degree)/self.max_degree_rotation) + self.min_duty;

        if  duty > self.max_duty { duty = self.max_duty; }
        else if duty < self.min_duty{ duty = self.min_duty; }

        return duty;
    }

    pub fn duty_to_degree(&mut self, duty: u32) -> u32 {
        let degree = ((duty - self.min_duty) * self.max_degree_rotation)/ (self.max_duty - self.min_duty);
        return degree;
    }

    pub fn rotate(&mut self, degree: u32) {
        let duty = self.degree_to_duty(degree);
        
        self.set_current_pos(degree);
        self.pwm.set_duty_cycle(duty as u16).unwrap();
    }

    pub fn rotate_duty(&mut self,mut  duty: u32) {

        if  duty > self.max_duty { duty = self.max_duty; }
        else if duty < self.min_duty{ duty = self.min_duty; }

        let current_degree = self.duty_to_degree(duty);
        self.set_current_pos(current_degree);

        self.pwm.set_duty_cycle(duty as u16).unwrap();
    }
}
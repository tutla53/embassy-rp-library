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
    embassy_time::Timer,
    fixed::{types::extra::U4, FixedU16},
    {defmt_rtt as _, panic_probe as _},
};

const DEFAULT_SERVO_FREQ: u32 = 50; //Hertz
const DEFAULT_MIN_PULSE_WIDTH: u32 = 1000; //us 
const DEFAULT_MAX_PULSE_WIDTH: u32 = 2000;  //us
const DEFAULT_MAX_DEGREE_ROTATION: u8 = 180; //degree
const DEFAULT_INITIAL_POSITION: u16 = 0; //degree

pub struct ServoBuilder<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    freq: u32, 
    min_pulse_width: u32,
    max_pulse_width: u32,
    max_degree_rotation: u8,
    initial_position: u16,
}

impl<'d> ServoBuilder<'d> {
    pub fn new(pwm: Pwm<'d>) -> Self {
        Self {
            pwm,
            cfg: Config::default(),
            freq: DEFAULT_SERVO_FREQ,
            min_pulse_width: DEFAULT_MIN_PULSE_WIDTH,
            max_pulse_width: DEFAULT_MAX_PULSE_WIDTH,
            max_degree_rotation: DEFAULT_MAX_DEGREE_ROTATION,
            initial_position: DEFAULT_INITIAL_POSITION,
        }
    }

    pub fn set_servo_freq(mut self, freq: u32) -> Self {
        self.freq = freq;
        self
    }

    pub fn set_min_pulse_width(mut self, duration: u32) -> Self {
        self.min_pulse_width = duration;
        self
    }

    pub fn set_max_pulse_width(mut self, duration: u32) -> Self {
        self.max_pulse_width = duration;
        self
    }

    pub fn set_max_degree_rotation(mut self, degree: u8) -> Self {
        self.max_degree_rotation = degree;
        self
    }

    pub fn set_initial_position(mut self, init_pos: u16) -> Self {
        self.initial_position = init_pos;
        self
    }

    pub fn build(mut self) -> Servo<'d> {
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 125u8;
        let period = (clock_freq_hz / (self.freq * divider as u32)) as u16 - 1;
        let num = FixedU16::<U4>::from_num(self.max_pulse_width - self.min_pulse_width);
        
        self.cfg.top = period;
        self.cfg.divider = divider.into();
        self.pwm.set_config(&self.cfg);

        Servo {
            pwm: self.pwm,
            cfg: self.cfg,
            min_pulse_width: FixedU16::<U4>::from_num(self.min_pulse_width),
            max_pulse_width: FixedU16::<U4>::from_num(self.max_pulse_width),
            max_degree_rotation:  FixedU16::<U4>::from_num(self.max_degree_rotation),
            current_pos: self.initial_position,
            numerator: num,
        }
    }
}

pub struct Servo<'d> {
    pwm: Pwm<'d>,
    cfg: Config,
    min_pulse_width: FixedU16<U4>,
    max_pulse_width: FixedU16<U4>,
    max_degree_rotation: FixedU16<U4>,
    current_pos: u16,
    numerator: FixedU16<U4>,
}

#[allow(dead_code)]
impl<'d> Servo<'d> {
    fn set_current_pos(&mut self, degree: u16){
        self.current_pos = degree;
    }

    pub fn get_current_pos(&mut self) -> u16 {
        return self.current_pos;
    }

    pub fn enable(&mut self) {
        self.cfg.enable = true;
        self.pwm.set_config(&self.cfg);
    }

    pub fn disable(&mut self) {
        self.cfg.enable = false;
        self.pwm.set_config(&self.cfg);
    }

    pub fn rotate(&mut self, degree: u16) {

        let mut duration = ((FixedU16::<U4>::from_num(degree)/self.max_degree_rotation) * self.numerator) + self.min_pulse_width;
        
        if  duration > self.max_pulse_width {
            duration = self.max_pulse_width;
        }
        
        let duration_int = duration.to_num::<u16>();

        self.set_current_pos(degree);
        self.pwm.set_duty_cycle(duration_int).unwrap();
    }

    pub async fn sweep(&mut self, target: u16, delay_ms: u64){
        let mut inc: i16 = 1;
        if self.current_pos > target {inc = -1}
    
        while self.current_pos != target {
            let mut new_pos = self.current_pos as i16 + inc;
            
            if new_pos<0 {new_pos = 0;}
            else if new_pos>180{new_pos = 180;}
    
            self.rotate(new_pos as u16);
            Timer::after_millis(delay_ms).await;
        }
    }

    fn rotate_duration(&mut self, duration: u16) {
        self.set_current_pos(duration.into());
        self.pwm.set_duty_cycle(duration).unwrap();
    }
}